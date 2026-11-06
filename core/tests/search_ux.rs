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
        let root = std::env::temp_dir().join(format!(
            "devkit-search-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed),
        ));
        let fixture = Self(root);
        let catalog = fixture.0.join("catalog");
        let mut packs = Vec::new();
        for (id, name, stack, version, documents) in [
            (
                "go", "Go", "go", "1.27",
                json!([
                    doc("std/net/http", "net/http", "The package contains HTTP examples. See http.Handler for details.", &["http.Handler"]),
                    doc("std/net/http::part-2", "net/http (part 2)", "Reading requests\n\nfunc (c *CrossOriginProtection) Handler(h Handler) Handler\n\ntype Handler interface {\n    ServeHTTP(ResponseWriter, *Request)\n}", &[]),
                    doc("other", "Other APIs", "Miscellaneous Go API notes.", &[]),
                ]),
            ),
            (
                "chi", "Chi", "go", "5.2.4",
                json!([
                    doc("api", "Chi Router", "Example: func Chain(handlers ...http.Handler) Middleware.", &["Chi Router"]),
                    doc("README.md", "<img alt=\"chi\" src=\"logo.svg\" />", "A lightweight router for Go.", &["Chi landing"]),
                    doc("example", "<em>Methods</em> for List<E>", "An example with both styled words and language syntax.", &["Methods example"]),
                ]),
            ),
            (
                "python", "Python", "python", "3.15",
                json!([
                    doc("library/pathlib.html", "pathlib", "Use pathlib.Path.read_text to load a file.\n\n class pathlib.Path(*pathsegments)\n\n Path.read_text(encoding=None, errors=None, newline=None)\n\n Return the decoded contents.", &["pathlib.Path", "pathlib.Path.read_text"]),
                ]),
            ),
            (
                "fastapi", "FastAPI", "python", "0.162",
                json!([
                    doc("index.md", "FastAPI", "The project landing page.", &[]),
                    doc("api/FastAPI", "FastAPI", "class FastAPI(Starlette)\n\nBuild API applications.", &[]),
                    doc("tutorial.md", "Tutorial", "# Tutorial\n\n```python\nfirst = 1", &[]),
                    doc("tutorial.md::part-2", "Tutorial (part 2)", "second = 2\n\n```\n\n## **Next** steps\n\n`second` is available.", &[]),
                ]),
            ),
            (
                "elixir", "Elixir", "elixir", "1.20",
                json!([
                    doc("Enum.html", "Enum", "@spec frequencies(t()) :: map()\n\nmap(enumerable, fun)\n\n@spec map(t(), (element() -> any())) :: list()\n\nTransforms every element.", &["Enum.map/2"]),
                ]),
            ),
        ] {
            let path = format!("docs/{id}");
            fs::create_dir_all(catalog.join(&path)).unwrap();
            fs::write(
                catalog.join(&path).join("documents.json"),
                serde_json::to_vec(&documents).unwrap(),
            ).unwrap();
            packs.push(json!({"id":id,"name":name,"stack":stack,"version":version,"description":"Fixture","source_url":"https://example.org","license":"MIT","path":path}));
        }
        fs::write(
            catalog.join("catalog.json"),
            serde_json::to_vec(&json!({"schema_version":1,"docs":packs,"templates":[]})).unwrap(),
        )
        .unwrap();
        for pack in ["go", "chi", "python", "fastapi", "elixir"] {
            fixture
                .call(json!({"command":"docs.install","id":pack}))
                .unwrap();
        }
        fixture
    }

    fn call(&self, mut request: Value) -> anyhow::Result<Value> {
        request["home"] = json!(self.0.join("library"));
        request["catalog"] = json!(self.0.join("catalog"));
        execute(&request)
    }
}

#[test]
fn renders_installed_markdown_and_preserves_plain_runtime_docs() {
    let fixture = Fixture::new();
    let markdown = fixture
        .call(
            json!({"command":"docs.show","id":"fastapi@0.162/tutorial.md::part-2","query":"Next"}),
        )
        .unwrap();
    let html = markdown["html"].as_str().unwrap();
    assert!(html.contains("<pre><code class=\"language-python\">second = 2"));
    assert!(html.contains("<h2 id=\"next-steps\"><strong>Next</strong> steps</h2>"));
    assert_eq!(markdown["focus_text"], "Next steps");
    assert_eq!(
        markdown["body"],
        "second = 2\n\n```\n\n## **Next** steps\n\n`second` is available."
    );
    assert_eq!(markdown["source_url"], "https://example.org");

    let plain = fixture
        .call(json!({"command":"docs.show","id":"http.Handler","stack":"go"}))
        .unwrap();
    assert!(plain["html"].is_null());
    assert_eq!(plain["focus_text"], "type Handler interface {");
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn doc(id: &str, title: &str, body: &str, aliases: &[&str]) -> Value {
    json!({"id":id,"title":title,"body":body,"source_url":"https://example.org","aliases":aliases})
}

#[test]
fn exact_alias_prefers_declaration_in_related_continuation() {
    let fixture = Fixture::new();
    let results = fixture
        .call(json!({"command":"docs.search","query":"http.Handler","stack":"go"}))
        .unwrap();
    let matches = results.as_array().unwrap();
    assert_eq!(matches[0]["id"], "go@1.27/std/net/http::part-2");
    assert_eq!(matches[0]["match_text"], "type Handler interface {");
    assert!(matches[0]["snippet"]
        .as_str()
        .unwrap()
        .starts_with("type Handler interface {"));
    assert!(matches
        .iter()
        .all(|hit| hit["id"] != "go@1.27/std/net/http"));

    let alias = fixture
        .call(json!({"command":"docs.show","id":"http.Handler"}))
        .unwrap();
    assert_eq!(alias["id"], matches[0]["id"]);
    assert_eq!(alias["focus_text"], matches[0]["match_text"]);

    let original = fixture
        .call(json!({"command":"docs.show","id":"go@1.27/std/net/http","query":"http.Handler"}))
        .unwrap();
    assert_eq!(original["id"], "go@1.27/std/net/http");
}

#[test]
fn prefix_precedes_body_mentions_and_api_wins_an_exact_tie() {
    let fixture = Fixture::new();
    let results = fixture
        .call(json!({"command":"docs.search","query":"http.H","stack":"go","limit":2}))
        .unwrap();
    assert_eq!(results[0]["pack_id"], "go");
    assert_eq!(results[1]["pack_id"], "chi");

    let results = fixture
        .call(json!({"command":"docs.search","query":"FastAPI","stack":"python"}))
        .unwrap();
    assert_eq!(results[0]["id"], "fastapi@0.162/api/FastAPI");
    assert_eq!(results[1]["id"], "fastapi@0.162/index.md");
}

#[test]
fn qualified_method_anchors_on_its_signature() {
    let fixture = Fixture::new();
    let results = fixture
        .call(json!({"command":"docs.search","query":"pathlib.Path.read_text"}))
        .unwrap();
    let anchor = "Path.read_text(encoding=None, errors=None, newline=None)";
    assert_eq!(results[0]["match_text"], anchor);
    assert!(results[0]["snippet"].as_str().unwrap().starts_with(anchor));
    let read = fixture
        .call(json!({"command":"docs.show","id":results[0]["id"],"query":"pathlib.Path.read_text"}))
        .unwrap();
    assert_eq!(read["focus_text"], anchor);
    let elixir = fixture
        .call(json!({"command":"docs.show","id":"Enum.map/2"}))
        .unwrap();
    assert_eq!(
        elixir["focus_text"],
        "@spec map(t(), (element() -> any())) :: list()"
    );
}

#[test]
fn scopes_and_paging_apply_to_browse_search_and_read() {
    let fixture = Fixture::new();
    let first = fixture
        .call(json!({"command":"docs.search","query":"","pack_id":"chi","limit":1}))
        .unwrap();
    let next = fixture
        .call(
            json!({"command":"docs.search","pack_id":"chi","version":"5.2.4","limit":1,"offset":1}),
        )
        .unwrap();
    assert_eq!(first[0]["pack_id"], "chi");
    assert_eq!(next[0]["pack_id"], "chi");
    assert_ne!(first[0]["id"], next[0]["id"]);
    assert_eq!(
        fixture
            .call(json!({"command":"docs.search","pack_id":"chi","offset":999_999_999u64}))
            .unwrap(),
        json!([])
    );

    let results = fixture
        .call(json!({"command":"docs.search","query":"http.Handler","pack_id":"chi","offset":1}))
        .unwrap();
    assert_eq!(results[0]["pack_id"], "chi");
    assert!(fixture
        .call(json!({"command":"docs.show","id":"go@1.27/std/net/http","pack_id":"chi"}))
        .is_err());
    assert_eq!(
        fixture
            .call(json!({"command":"docs.search","query":"Chi","pack_id":"go"}))
            .unwrap(),
        json!([])
    );
    assert_eq!(
        fixture
            .call(json!({"command":"docs.search","pack_id":"chi","version":"1.27"}))
            .unwrap(),
        json!([])
    );
}

#[test]
fn reader_and_results_hide_presentational_title_tags() {
    let fixture = Fixture::new();
    let results = fixture
        .call(json!({"command":"docs.search","query":"Chi landing"}))
        .unwrap();
    assert_eq!(results[0]["title"], "Chi");
    let read = fixture
        .call(json!({"command":"docs.show","id":results[0]["id"]}))
        .unwrap();
    assert_eq!(read["title"], "Chi");
    let example = fixture
        .call(json!({"command":"docs.show","id":"Methods example"}))
        .unwrap();
    assert_eq!(example["title"], "Methods for List<E>");
}
