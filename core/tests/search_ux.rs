use devline_core::execute;
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
            "devline-search-{}-{}-{}",
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
                    doc_at("std/net/http", "net/http", "The package contains HTTP examples. See http.Handler for details.", &["http.Handler"], "https://example.org/net/http"),
                    doc_at("std/net/http::part-2", "net/http (part 2)", "Reading requests\n\nfunc (c *CrossOriginProtection) Handler(h Handler) Handler\n\ntype Handler interface {\n    ServeHTTP(ResponseWriter, *Request)\n}", &[], "https://example.org/net/http"),
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
                    doc_at("library/pathlib.html", "pathlib", "Use pathlib.Path.read_text to load a file.\n\n class pathlib.Path(*pathsegments)\n\n Path.read_text(encoding=None, errors=None, newline=None)\n\n Return the decoded contents.", &["pathlib.Path", "pathlib.Path.read_text"], "https://example.org/library/pathlib.html"),
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
                "aspnet", "ASP.NET Core", "dotnet", "10",
                json!([
                    html_at("iis/index", "IIS", "<h1 id=\"iis\">IIS</h1>", "https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/iis/index"),
                    html_at("iis/advanced", "Advanced IIS", "<h2 id=\"iis-configuration\">IIS configuration</h2>", "https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/iis/advanced"),
                    html_at("iis/hosting-bundle", "Hosting Bundle", "<h2 id=\"install-the-net-hosting-bundle\">Install the .NET Hosting Bundle</h2>", "https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/iis/hosting-bundle"),
                    html_at("security/enforcing-ssl", "HTTPS", "<h2 id=\"http-strict-transport-security-hsts-protocol\">HSTS</h2><h2 id=\"Options&lt;T&gt;\">Options</h2><a name=\"legacy\"></a><pre>&lt;h2 id=\"missing\"&gt;</pre>", "https://learn.microsoft.com/en-us/aspnet/core/security/enforcing-ssl"),
                ]),
            ),
            (
                "ruby", "Ruby", "ruby", "4",
                json!([
                    html_at("api/Array", "Array", "<h1 id=\"array\">Array</h1>", "https://docs.ruby-lang.org/en/4.0/Array.html"),
                    html_at("method/Array#map", "Array#map", "<h1 id=\"arraymap\">Array#map</h1>", "https://docs.ruby-lang.org/en/4.0/Array.html#method-i-map"),
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
        for pack in ["go", "chi", "python", "fastapi", "aspnet", "ruby", "elixir"] {
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
    doc_at(id, title, body, aliases, "https://example.org")
}

fn doc_at(id: &str, title: &str, body: &str, aliases: &[&str], source_url: &str) -> Value {
    json!({"id":id,"title":title,"body":body,"source_url":source_url,"aliases":aliases})
}

fn html_at(id: &str, title: &str, html: &str, source_url: &str) -> Value {
    let mut article = doc_at(id, title, title, &[], source_url);
    article["html"] = json!(html);
    article
}

#[test]
fn indexed_suffixes_and_callable_syntax_work_after_legacy_backfill() {
    let fixture = Fixture::new();
    let db = rusqlite::Connection::open(fixture.0.join("library/library.sqlite3")).unwrap();
    db.execute_batch("DELETE FROM doc_symbol_keys; DELETE FROM doc_symbol_packs;")
        .unwrap();
    drop(db);
    let cases = [
        "Path.read_text",
        "pathlib.Path.read_text()",
        "py:Path.read_text",
        "go,python@3.15:Path.read_text",
    ];
    for query in cases {
        let results = fixture
            .call(json!({"command":"docs.search","query":query,"limit":3}))
            .unwrap();
        let first = &results[0];
        assert_eq!(first["id"], "python@3.15/library/pathlib.html", "{query}");
        assert_eq!(first["symbol"], "pathlib.Path.read_text", "{query}");
        assert_eq!(first["match_kind"], "symbol");
        assert_eq!(
            first["match_text"],
            "Path.read_text(encoding=None, errors=None, newline=None)"
        );
    }
    let direct = fixture
        .call(
            json!({"command":"docs.show","id":"py:pathlib.Path.read_text()","project_stack":"go"}),
        )
        .unwrap();
    assert_eq!(direct["id"], "python@3.15/library/pathlib.html");
    let excluded = fixture
        .call(json!({"command":"docs.search","query":"py:Path.read_text","stack":"go"}))
        .unwrap();
    assert_eq!(excluded, json!([]));
}

#[test]
fn resolves_offline_links_and_keeps_the_originating_release() {
    let fixture = Fixture::new();
    let target = fixture.call(json!({"command":"docs.resolve","url":"https://example.org/net/http/#Handler","id":"go@1.27/std/net/http"})).unwrap();
    assert_eq!(target["id"], "go@1.27/std/net/http::part-2");
    assert_eq!(target["anchor"], "Handler");
    assert_eq!(target["focus_text"], "type Handler interface {");
    let next = fixture
        .call(json!({"command":"docs.show","id":"go@1.27/std/net/http"}))
        .unwrap();
    assert_eq!(next["next_id"], target["id"]);
    assert_eq!(next["part"], 1);
    assert_eq!(next["parts"], 2);
    let previous = fixture
        .call(json!({"command":"docs.show","id":target["id"]}))
        .unwrap();
    assert_eq!(previous["previous_id"], "go@1.27/std/net/http");
    let cross = fixture.call(json!({"command":"docs.resolve","url":"../library/pathlib.html#pathlib.Path.read_text","id":"go@1.27/std/net/http"})).unwrap();
    assert_eq!(cross["id"], "python@3.15/library/pathlib.html");
    assert_eq!(
        cross["focus_text"],
        "Path.read_text(encoding=None, errors=None, newline=None)"
    );
    for request in [
        json!({"command":"docs.resolve","url":"https://example.org/net/http#Handler","pack_id":"go","version":"1.30"}),
        json!({"command":"docs.resolve","url":"https://example.org/net/http?version=1.30#Handler"}),
        json!({"command":"docs.resolve","url":"file:///etc/passwd"}),
    ] {
        assert!(fixture.call(request).unwrap().is_null());
    }
}

#[test]
fn html_links_land_on_exact_or_verified_sections() {
    let fixture = Fixture::new();
    let origin = "aspnet@10/security/enforcing-ssl";
    let resolve = |url| {
        fixture
            .call(json!({"command":"docs.resolve","url":url,"id":origin}))
            .unwrap()
    };
    for (url, id, anchor) in [
        ("#Options%3CT%3E", origin, "Options<T>"),
        (
            "#hsts",
            origin,
            "http-strict-transport-security-hsts-protocol",
        ),
        (
            "../host-and-deploy/iis/index#install-the-net-hosting-bundle",
            "aspnet@10/iis/hosting-bundle",
            "install-the-net-hosting-bundle",
        ),
    ] {
        assert_eq!(
            resolve(url),
            json!({"id":id,"anchor":anchor,"focus_text":null})
        );
    }
    for url in [
        "#missing",
        "#%00",
        "?view=aspnetcore-11.0#hsts",
        "https://example.org/uninstalled#heading",
        "../host-and-deploy/iis/index#http2-support",
    ] {
        assert_eq!(resolve(url), Value::Null, "{url}");
    }
    assert_eq!(
        resolve("#"),
        json!({"id":origin,"anchor":null,"focus_text":null})
    );
}

#[test]
fn link_repair_uses_the_origin_revision_and_preserves_exact_anchors() {
    let fixture = Fixture::new();
    let catalog = fixture.0.join("catalog");
    let manifest = catalog.join("catalog.json");
    let mut content: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    let aspnet = content["docs"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|pack| pack["id"] == "aspnet")
        .unwrap();
    aspnet["version"] = json!("10+devline.1");
    fs::write(&manifest, serde_json::to_vec(&content).unwrap()).unwrap();
    let path = catalog.join("docs/aspnet/documents.json");
    let mut articles: Vec<Value> = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    articles.retain(|article| article["id"] != "iis/advanced");
    let iis = articles
        .iter_mut()
        .find(|article| article["id"] == "iis/index")
        .unwrap();
    iis["html"] = json!("<h2 id=\"install-the-net-hosting-bundle\">Install here</h2>");
    fs::write(path, serde_json::to_vec(&articles).unwrap()).unwrap();
    fixture
        .call(json!({"command":"docs.install","id":"aspnet"}))
        .unwrap();

    let resolve = |version, url| {
        fixture.call(json!({
            "command":"docs.resolve","url":url,"id":format!("aspnet@{version}/security/enforcing-ssl")
        })).unwrap()
    };
    let url = "../host-and-deploy/iis/index#install-the-net-hosting-bundle";
    assert_eq!(resolve("10", url)["id"], "aspnet@10/iis/hosting-bundle");
    assert_eq!(
        resolve("10+devline.1", url)["id"],
        "aspnet@10+devline.1/iis/index"
    );
    let moved = "../host-and-deploy/iis/index#iis-configuration";
    assert_eq!(resolve("10", moved)["id"], "aspnet@10/iis/advanced");
    assert_eq!(resolve("10+devline.1", moved), Value::Null);
}

#[test]
fn fragment_addressed_article_opens_without_a_phantom_anchor() {
    let fixture = Fixture::new();
    let resolved = fixture
        .call(json!({
            "command":"docs.resolve",
            "id":"ruby@4/api/Array",
            "url":"#method-i-map"
        }))
        .unwrap();
    assert_eq!(
        resolved,
        json!({"id":"ruby@4/method/Array#map","anchor":null,"focus_text":null})
    );
}

#[test]
fn revision_heads_are_default_but_retained_versions_stay_addressable() {
    let fixture = Fixture::new();
    let catalog = fixture.0.join("catalog/catalog.json");
    let mut content: Value = serde_json::from_slice(&fs::read(&catalog).unwrap()).unwrap();
    let python = content["docs"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|doc| doc["id"] == "python")
        .unwrap();
    python["version"] = json!("3.15+devline.1");
    fs::write(&catalog, serde_json::to_vec(&content).unwrap()).unwrap();
    let pending = fixture.call(json!({"command":"updates.status"})).unwrap();
    let update = pending["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "docs:python")
        .unwrap();
    assert_eq!(update["reference"], "python@3.15+devline.1");
    fixture
        .call(json!({"command":"docs.install","id":"python"}))
        .unwrap();
    let default = fixture
        .call(json!({"command":"docs.search","query":"py:Path.read_text"}))
        .unwrap();
    assert_eq!(default[0]["version"], "3.15+devline.1");
    assert!(default
        .as_array()
        .unwrap()
        .iter()
        .all(|hit| hit["version"] == "3.15+devline.1"));
    let pinned = fixture
        .call(json!({"command":"docs.search","query":"python@3.15:Path.read_text"}))
        .unwrap();
    assert_eq!(pinned[0]["version"], "3.15");
    let catalog = fixture.call(json!({"command":"catalog"})).unwrap();
    assert_eq!(
        catalog["docs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|doc| doc["id"] == "python" && doc["installed"] == true)
            .count(),
        2
    );
    let url = "https://example.org/library/pathlib.html#pathlib.Path.read_text";
    let from_old = fixture
        .call(json!({"command":"docs.resolve","url":url,"id":"python@3.15/library/pathlib.html"}))
        .unwrap();
    assert_eq!(from_old["id"], "python@3.15/library/pathlib.html");
    let unpinned = fixture
        .call(json!({"command":"docs.resolve","url":url}))
        .unwrap();
    assert_eq!(unpinned["id"], "python@3.15+devline.1/library/pathlib.html");
}

#[test]
fn sharded_pack_import_is_atomic_and_immutable() {
    let fixture = Fixture::new();
    let catalog = fixture.0.join("catalog/catalog.json");
    let mut content: Value = serde_json::from_slice(&fs::read(&catalog).unwrap()).unwrap();
    let python = content["docs"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|doc| doc["id"] == "python")
        .unwrap();
    python["version"] = json!("3.15+devline.1");
    python["shards"] = json!(2);
    fs::write(&catalog, serde_json::to_vec(&content).unwrap()).unwrap();
    let request = json!({"command":"docs.install","id":"python","version":"3.15+devline.1"});
    assert!(fixture.call(request.clone()).is_err());
    let search = json!({"command":"docs.search","query":"python@3.15+devline.1:Path.read_text"});
    assert_eq!(fixture.call(search.clone()).unwrap(), json!([]));
    let shard = fixture.0.join("catalog/docs/python/documents-2.json");
    fs::write(&shard, "[]").unwrap();
    assert!(fixture.call(request.clone()).is_err());
    assert_eq!(fixture.call(search.clone()).unwrap(), json!([]));
    let second = json!([doc(
        "library/strings.html",
        "Strings",
        "The string guide.",
        &["StrGuide"]
    )]);
    fs::write(&shard, serde_json::to_vec(&second).unwrap()).unwrap();
    assert_eq!(fixture.call(request.clone()).unwrap()["count"], 2);
    assert_eq!(fixture.call(request.clone()).unwrap()["count"], 2);
    assert_eq!(
        fixture.call(search).unwrap()[0]["version"],
        "3.15+devline.1"
    );
    fs::write(&shard, serde_json::to_vec_pretty(&second).unwrap()).unwrap();
    assert!(fixture.call(request).is_err());
}

#[test]
fn category_filters_apply_to_browse_search_and_legacy_rows() {
    let fixture = Fixture::new();
    let db = rusqlite::Connection::open(fixture.0.join("library/library.sqlite3")).unwrap();
    db.execute_batch("DROP INDEX documents_category; ALTER TABLE documents DROP COLUMN category;")
        .unwrap();
    drop(db);
    let all = fixture
        .call(json!({"command":"docs.search","pack_id":"go","query":""}))
        .unwrap();
    assert_eq!(all.as_array().unwrap().len(), 2);
    let api = fixture
        .call(json!({"command":"docs.search","pack_id":"go","query":"","category":"api"}))
        .unwrap();
    assert_eq!(api.as_array().unwrap().len(), 1);
    assert_eq!(api[0]["id"], "go@1.27/std/net/http");
    assert_eq!(api[0]["category"], "api");
    let declaration = fixture
        .call(json!({"command":"docs.search","query":"http.Handler","category":"api"}))
        .unwrap();
    assert_eq!(declaration[0]["id"], "go@1.27/std/net/http::part-2");
    let excluded = fixture.call(json!({"command":"docs.search","pack_id":"go","query":"http.Handler","category":"guide"})).unwrap();
    assert_eq!(excluded, json!([]));
    let reference = fixture.call(json!({"command":"docs.search","pack_id":"go","query":"miscellaneous","category":"reference"})).unwrap();
    assert_eq!(reference[0]["id"], "go@1.27/other");
    assert_eq!(reference[0]["category"], "reference");
    assert!(fixture
        .call(json!({"command":"docs.search","query":"","category":"unknown"}))
        .is_err());
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
