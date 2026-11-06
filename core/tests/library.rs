use devkit_core::execute;
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "devkit-test-{}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("catalog/docs/go")).unwrap();
        fs::write(root.join("catalog/catalog.json"),json!({"schema_version":1,"docs":[{"id":"go","name":"Go","stack":"go","version":"1.0","description":"fixture","source_url":"https://go.dev","license":"BSD","path":"docs/go"}],"templates":[]}).to_string()).unwrap();
        fs::write(root.join("catalog/docs/go/documents.json"),json!([
            {"id":"handler","title":"http.Handler","body":"ServeHTTP handles a request. This is the HTTP interface.","source_url":"https://pkg.go.dev/net/http#Handler","aliases":["Handler"]},
            {"id":"client","title":"http.Client","body":"An HTTP client sends a request and reads its response.","source_url":"https://pkg.go.dev/net/http#Client","aliases":[]}
        ]).to_string()).unwrap();
        Self { root }
    }
    fn call(&self, mut v: Value) -> anyhow::Result<Value> {
        v["home"] = json!(self.root.join("library"));
        v["catalog"] = json!(self.root.join("catalog"));
        execute(&v)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
#[test]
fn install_search_read_and_remove_offline() {
    let f = Fixture::new();
    assert_eq!(
        f.call(json!({"command":"docs.install","id":"go"})).unwrap()["count"],
        2
    );
    assert_eq!(
        f.call(json!({"command":"docs.show","id":"Handler"}))
            .unwrap()["title"],
        "http.Handler"
    );
    let exact = f
        .call(json!({"command":"docs.search","query":"http.Handler"}))
        .unwrap();
    assert_eq!(exact[0]["title"], "http.Handler");
    assert_eq!(
        f.call(json!({"command":"docs.search","query":"request","stack":"go"}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        f.call(json!({"command":"docs.search","query":"request","stack":"python"}))
            .unwrap(),
        json!([])
    );
    let doc = f
        .call(json!({"command":"docs.show","id":exact[0]["id"]}))
        .unwrap();
    assert!(doc["body"].as_str().unwrap().contains("ServeHTTP"));
    f.call(json!({"command":"docs.remove","id":"go","version":"1.0"}))
        .unwrap();
    assert_eq!(
        f.call(json!({"command":"docs.search","query":"request"}))
            .unwrap(),
        json!([])
    );
}
#[test]
fn failed_import_rolls_back_and_installed_version_is_immutable() {
    let f = Fixture::new();
    f.call(json!({"command":"docs.install","id":"go"})).unwrap();
    let path = f.root.join("catalog/docs/go/documents.json");
    let original = fs::read(&path).unwrap();
    fs::write(&path, b"[]").unwrap();
    assert!(f.call(json!({"command":"docs.install","id":"go"})).is_err());
    assert_eq!(
        f.call(json!({"command":"docs.search","query":"Handler"}))
            .unwrap()[0]["title"],
        "http.Handler"
    );
    fs::write(path, original).unwrap();
    assert!(f.call(json!({"command":"docs.install","id":"go"})).is_ok());
}
#[test]
fn literal_search_cannot_inject_fts_operators() {
    let f = Fixture::new();
    f.call(json!({"command":"docs.install","id":"go"})).unwrap();
    for term in ["C++", "std::mem", "\" OR body:*", "(", "x OR y"] {
        assert!(
            f.call(json!({"command":"docs.search","query":term}))
                .is_ok(),
            "{term}"
        );
    }
}

#[test]
fn concurrent_installs_publish_one_complete_search_index() {
    let f = Fixture::new();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| f.call(json!({"command":"docs.install","id":"go"})).unwrap()))
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap()["count"], 2);
        }
    });
    assert_eq!(
        f.call(json!({"command":"docs.search","query":"request"}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn duplicate_document_id_rolls_back_whole_pack() {
    let f = Fixture::new();
    let path = f.root.join("catalog/docs/go/documents.json");
    let mut docs: Vec<Value> = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    docs.push(docs[0].clone());
    fs::write(&path, serde_json::to_vec(&docs).unwrap()).unwrap();
    assert!(f.call(json!({"command":"docs.install","id":"go"})).is_err());
    assert_eq!(
        f.call(json!({"command":"docs.search","query":"request"}))
            .unwrap(),
        json!([])
    );
    assert_eq!(
        f.call(json!({"command":"catalog"})).unwrap()["docs"][0]["installed"],
        false
    );
}

#[test]
fn prepare_uses_docs_from_selected_release() {
    let f = Fixture::new();
    let path = f.root.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut releases = Vec::new();
    for (version, docs) in [("1.0.0", json!(["missing"])), ("2.0.0", json!(["go@1.0"]))] {
        let relative = format!("templates/go/test/{version}");
        let project = f.root.join("catalog").join(&relative).join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("README.md"), "Fixture").unwrap();
        releases.push(json!({"id":"go/test","name":"Fixture","stack":"go","version":version,"description":"fixture","app_type":"cli","path":relative,"toolchain":"devkit-test-missing-runtime","minimum_version":"1.0.0","test_command":"true","docs":docs,"verified":true,"target":"any"}));
    }
    catalog["templates"] = json!(releases);
    fs::write(path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let prepared = f
        .call(json!({"command":"prepare","id":"go/test@2.0.0"}))
        .unwrap();
    assert_eq!(prepared["template"]["version"], "2.0.0");
    assert_eq!(prepared["docs"][0]["version"], "1.0");
}
