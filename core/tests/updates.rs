use devkit_core::execute;
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "devkit-update-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed),
        ));
        let fixture = Self(path);
        let catalog = fixture.0.join("catalog");
        let mut docs = Vec::new();
        let mut templates = Vec::new();
        for (doc_version, starter_version) in [("1.27.2", "1.0.0"), ("1.27.3", "1.1.0")] {
            let doc_path = format!("docs/go/{doc_version}");
            fs::create_dir_all(catalog.join(&doc_path)).unwrap();
            fs::write(catalog.join(&doc_path).join("documents.json"), json!([{
                "id":"intro","title":"Go","body":"Official Go reference","source_url":"https://go.dev/doc/"
            }]).to_string()).unwrap();
            docs.push(json!({"id":"go","name":"Go","stack":"go","version":doc_version,
                "description":"Go reference","source_url":"https://go.dev/doc/","license":"BSD-3-Clause","path":doc_path}));
            let starter_path = format!("templates/go/chi-api/{starter_version}");
            let project = catalog.join(&starter_path).join("project");
            fs::create_dir_all(&project).unwrap();
            fs::write(
                project.join("go.mod"),
                "module example\nrequire github.com/go-chi/chi/v5 v5.2.4\n",
            )
            .unwrap();
            templates.push(
                json!({"id":"go/chi-api","name":"Go API","stack":"go","version":starter_version,
                "description":"Go project","app_type":"api","path":starter_path,
                "toolchain":"devkit-test-absent","minimum_version":"1.27.3","test_command":"true",
                "docs":["go"],"verified":true,"target":"any"}),
            );
        }
        fs::write(
            catalog.join("catalog.json"),
            json!({"schema_version":1,"docs":docs,"templates":templates}).to_string(),
        )
        .unwrap();
        fixture
    }

    fn call(&self, mut request: Value) -> anyhow::Result<Value> {
        request["home"] = json!(self.0.join("library"));
        request["catalog"] = json!(self.0.join("catalog"));
        execute(&request)
    }

    fn item<'a>(result: &'a Value, id: &str) -> &'a Value {
        result["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn local_status_installs_exact_catalog_releases_and_retains_versions() {
    let fixture = Fixture::new();
    fixture
        .call(json!({"command":"docs.install","id":"go","version":"1.27.2"}))
        .unwrap();
    fixture
        .call(json!({"command":"templates.install","id":"go/chi-api@1.0.0"}))
        .unwrap();
    let snapshot = fixture.call(json!({"command":"updates.status"})).unwrap();
    assert!(snapshot["checked_at"].is_null());
    assert_eq!(Fixture::item(&snapshot, "app:devkit")["state"], "unchecked");
    assert!(Fixture::item(&snapshot, "app:devkit")["detail"]
        .as_str()
        .unwrap()
        .contains("github.com/fpl0/devline/releases/latest/download/release.json"));
    let docs = Fixture::item(&snapshot, "docs:go");
    assert_eq!(docs["action"], "install_docs");
    assert_eq!(docs["reference"], "go@1.27.3");
    let starter = Fixture::item(&snapshot, "template:go/chi-api");
    assert_eq!(starter["action"], "install_template");
    assert_eq!(starter["reference"], "go/chi-api@1.1.0");

    fixture
        .call(json!({"command":"docs.install","id":"go","version":"1.27.3"}))
        .unwrap();
    fixture
        .call(json!({"command":"templates.install","id":"go/chi-api@1.1.0"}))
        .unwrap();
    let snapshot = fixture.call(json!({"command":"updates.status"})).unwrap();
    let docs = Fixture::item(&snapshot, "docs:go");
    assert_eq!(docs["state"], "unchecked");
    assert_eq!(docs["current_version"], "1.27.3");
    assert!(docs["detail"].as_str().unwrap().contains("1.27.2, 1.27.3"));
    let starter = Fixture::item(&snapshot, "template:go/chi-api");
    assert_eq!(starter["state"], "unchecked");
    assert_eq!(starter["current_version"], "1.1.0");
    assert!(starter["detail"].as_str().unwrap().contains("1.1.0, 1.0.0"));
}

#[test]
fn app_channels_keep_cached_release_on_error_and_clear_on_reconfigure() {
    let fixture = Fixture::new();
    let invalid = fixture.call(
        json!({"command":"updates.configure","url":"https://name:secret@example.org/updates.json"}),
    );
    assert!(invalid.is_err());
    let configured = fixture
        .call(json!({"command":"updates.configure","url":"https://example.org/updates.json"}))
        .unwrap();
    assert_eq!(
        Fixture::item(&configured, "app:devkit")["state"],
        "unchecked"
    );
    fs::write(
        fixture.0.join("library/updates.json"),
        json!({
            "app_feed":"https://example.org/updates.json","checked_at":123,
            "observations":{"app":{"version":"9.0.0","compatible":null,
                "url":"https://example.org/releases/9.0.0","error":"Could not connect"}}
        })
        .to_string(),
    )
    .unwrap();
    let saved = fixture.call(json!({"command":"updates.status"})).unwrap();
    let app = Fixture::item(&saved, "app:devkit");
    assert_eq!(app["state"], "error");
    assert_eq!(app["available_version"], "9.0.0");
    assert_eq!(app["action"], "open_url");
    assert!(app["detail"].as_str().unwrap().contains("Last successful"));
    let cleared = fixture
        .call(json!({"command":"updates.configure","url":""}))
        .unwrap();
    assert_eq!(
        Fixture::item(&cleared, "app:devkit")["state"],
        "unconfigured"
    );
    let persisted = fixture.call(json!({"command":"updates.status"})).unwrap();
    assert_eq!(
        Fixture::item(&persisted, "app:devkit")["available_version"],
        Value::Null
    );
}
