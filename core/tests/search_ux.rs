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
        Self::with(vec![
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
                    doc("tutorial/request-files.md", "Request Files", "# Request Files\n\n## Uploading files\n\nUse an uploaded file to attach a document.", &[]),
                    doc("reference/forms.md", "Form Options", "A form can contain a file. Users upload values with a request.", &[]),
                ]),
            ),
            (
                "aspnet", "ASP.NET Core", "dotnet", "10",
                json!([
                    html_at("iis/index", "IIS", "<h1 id=\"iis\">IIS</h1>", "https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/iis/index"),
                    html_at("iis/advanced", "Advanced IIS", "<h2 id=\"iis-configuration\">IIS configuration</h2>", "https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/iis/advanced"),
                    html_at("iis/hosting-bundle", "Hosting Bundle", "<h2 id=\"install-the-net-hosting-bundle\">Install the .NET Hosting Bundle</h2>", "https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/iis/hosting-bundle"),
                    html_at("security/enforcing-ssl", "HTTPS", "<h2 id=\"http-strict-transport-security-hsts-protocol\">HSTS</h2><h2 id=\"Options&lt;T&gt;\">Options</h2><a name=\"legacy\"></a><pre>&lt;h2 id=\"missing\"&gt;</pre>", "https://learn.microsoft.com/en-us/aspnet/core/security/enforcing-ssl"),
                    doc("reference/sql-select", "PostgreSQL · SELECT", "The SQL command SELECT retrieves rows from a table.", &[]),
                    doc("reference/sql-functions", "PostgreSQL · Query Language (SQL) Functions", "SQL select queries can call functions. This chapter describes function parameters.", &[]),
                    html_at("reference/db-shell", "MongoDB Shell · Database methods", "<h2 id=\"example\">const cursor = db.sql(\"SELECT * FROM myCollection\", options)</h2><p>An example.</p>", "https://example.org/db-shell"),
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
        ])
    }

    /// Installs the given docsets, each `(id, name, stack, version, articles)`.
    fn with(docsets: Vec<(&str, &str, &str, &str, Value)>) -> Self {
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
        for (id, name, stack, version, documents) in &docsets {
            let path = format!("docs/{id}");
            fs::create_dir_all(catalog.join(&path)).unwrap();
            fs::write(
                catalog.join(&path).join("documents.json"),
                serde_json::to_vec(documents).unwrap(),
            )
            .unwrap();
            packs.push(json!({"id":id,"name":name,"stack":stack,"version":version,"description":"Fixture","source_url":"https://example.org","license":"MIT","path":path}));
        }
        fs::write(
            catalog.join("catalog.json"),
            serde_json::to_vec(&json!({"schema_version":1,"docs":packs,"templates":[]})).unwrap(),
        )
        .unwrap();
        for (pack, _, _, _, _) in &docsets {
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
fn reader_preflights_link_destinations_without_changing_the_legacy_resolver() {
    let fixture = Fixture::new();
    let origin = "aspnet@10/security/enforcing-ssl";
    let urls = [
        "#Options%3CT%3E",
        "#not-present",
        "https://outside.example/tutorial",
        "file:///etc/passwd",
        "#legacy",
        "enforcing-ssl#http-strict-transport-security-hsts-protocol",
        "enforcing-ssl#missing",
        "https://outside.example/tutorial#missing",
    ];
    let status = fixture
        .call(json!({"command":"docs.link-status","id":origin,"urls":urls}))
        .unwrap();
    assert_eq!(
        status[0],
        json!({"url":urls[0],"kind":"installed","id":origin,"anchor":"Options<T>","focus_text":null})
    );
    assert_eq!(status[1]["kind"], "missingSection");
    assert_eq!(status[2]["kind"], "online");
    assert_eq!(status[3]["kind"], "unavailable");
    assert_eq!(status[4]["anchor"], "legacy");
    assert_eq!(
        status[5]["anchor"],
        "http-strict-transport-security-hsts-protocol"
    );
    assert_eq!(status[6]["kind"], "missingSection");
    assert_eq!(status[7]["kind"], "online");
    assert_eq!(
        fixture
            .call(json!({"command":"docs.resolve","id":origin,"url":urls[1]}))
            .unwrap(),
        Value::Null
    );
}

#[test]
fn reader_reports_an_uninstalled_project_pin_instead_of_opening_another_release() {
    let fixture = Fixture::new();
    let project = fixture.0.join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("go.mod"),
        "module example.org/project\ngo 1.27\n",
    )
    .unwrap();
    let catalog_path = fixture.0.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    let mut available = catalog["docs"][0].clone();
    available["version"] = json!("1.27+devline.2");
    available["path"] = json!("docs/go/1.27+devline.2");
    catalog["docs"].as_array_mut().unwrap().push(available);
    fs::write(&catalog_path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    fixture
        .call(json!({"command":"project.pin","path":project,"id":"go","version":"1.27+devline.2"}))
        .unwrap();
    let url = "https://example.org/net/http#Handler";
    let result = fixture.call(json!({"command":"docs.link-status","id":"aspnet@10/security/enforcing-ssl","urls":[url],"project_path":project})).unwrap();
    assert_eq!(
        result[0],
        json!({"url":url,"kind":"missingDocument","pack_id":"go","version":"1.27+devline.2","can_install":true})
    );
}

#[test]
fn exact_topic_precedes_a_related_chapter_and_an_example_heading() {
    let fixture = Fixture::new();
    for limit in [5, 30] {
        let results = fixture
            .call(json!({"command":"docs.search","query":"sql select","limit":limit}))
            .unwrap();
        assert_eq!(results[0]["title"], "PostgreSQL · SELECT");
        let hits = results.as_array().unwrap();
        assert!(hits
            .iter()
            .any(|hit| hit["title"] == "PostgreSQL · Query Language (SQL) Functions"));
        assert!(hits.iter().any(|hit| hit["match_anchor"] == "example"));
    }
}

#[test]
fn imported_search_preserves_groups_and_exact_symbol_anchors() {
    let fixture = Fixture::new();
    let catalog_dir = fixture.0.join("catalog");
    let catalog_path = catalog_dir.join("catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    let id = "devdocs-go";
    let path = "docs/devdocs-go/1.27";
    catalog["docs"].as_array_mut().unwrap().push(json!({
        "id":id,"name":"DevDocs · Go","stack":id,"version":"1.27","description":"Imported fixture",
        "source_url":"https://devdocs.io/go/","license":"Fixture","path":path,"local_only":true,"entry_id":"index"
    }));
    fs::write(&catalog_path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    fs::create_dir_all(catalog_dir.join(path)).unwrap();
    let articles = json!([
        html_at("index", "Overview", "<h1>Overview</h1>", "https://devdocs.io/go/index"),
        {"id":"http","title":"HTTP package","body":"HTTP functions.","html":"<h2 id=\"handler-section\">Handlers</h2>",
         "source_url":"https://example.org/net/http","group":"Packages","aliases":["http.Handler"],
         "alias_anchors":{"http.Handler":"handler-section"}},
        {"id":"http::part-2","title":"HTTP continuation","body":"More HTTP functions.",
         "source_url":"https://example.org/net/http","group":"Continuations"}
    ]);
    fs::write(
        catalog_dir.join(path).join("documents.json"),
        serde_json::to_vec(&articles).unwrap(),
    )
    .unwrap();
    fixture
        .call(json!({"command":"docs.import.local","id":id,"version":"1.27"}))
        .unwrap();
    let navigation = fixture
        .call(json!({"command":"docs.navigation","pack_id":id,"version":"1.27"}))
        .unwrap();
    assert_eq!(
        navigation,
        json!({"groups":["Packages"],"has_ungrouped":true})
    );
    let group = fixture
        .call(json!({"command":"docs.search","pack_id":id,"group":"Packages"}))
        .unwrap();
    assert_eq!(group.as_array().unwrap().len(), 1);
    assert_eq!(group[0]["id"], "devdocs-go@1.27/http");
    let ungrouped = fixture
        .call(json!({"command":"docs.search","pack_id":id,"group":""}))
        .unwrap();
    assert_eq!(ungrouped.as_array().unwrap().len(), 1);
    assert_eq!(ungrouped[0]["id"], "devdocs-go@1.27/index");
    let page = fixture
        .call(json!({"command":"docs.search","pack_id":id,"limit":1,"offset":1}))
        .unwrap();
    assert_eq!(page[0]["id"], "devdocs-go@1.27/index");
    let result = fixture
        .call(json!({"command":"docs.search","kind":"imported","query":"http.Handler"}))
        .unwrap();
    assert_eq!(result[0]["symbol"], "http.Handler");
    assert_eq!(result[0]["match_anchor"], "handler-section");
    let curated = fixture
        .call(json!({"command":"docs.search","kind":"installed","query":"http.Handler"}))
        .unwrap();
    assert_eq!(curated[0]["pack_id"], "go");
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
fn topic_queries_rank_sections_and_keep_their_reader_location() {
    let fixture = Fixture::new();
    for query in ["fastapi:how do I upload a file", "fastapi:uplaoding fiels"] {
        let hits = fixture
            .call(json!({"command":"docs.search","query":query,"limit":1}))
            .unwrap();
        assert_eq!(
            hits[0]["id"], "fastapi@0.162/tutorial/request-files.md",
            "{query}"
        );
        assert_eq!(hits[0]["match_text"], "Uploading files");
        assert_eq!(hits[0]["match_anchor"], "uploading-files");
        assert_eq!(hits[0]["snippet"], "Uploading files");
    }

    let hits = fixture
        .call(json!({"command":"docs.search","query":"fastapi:next st","limit":1}))
        .unwrap();
    assert_eq!(hits[0]["id"], "fastapi@0.162/tutorial.md::part-2");
    assert_eq!(hits[0]["match_anchor"], "next-steps");
}

#[test]
fn qualified_symbol_typo_correction_stays_inside_the_requested_scope() {
    let fixture = Fixture::new();
    let hits = fixture
        .call(json!({"command":"docs.search","query":"go:htt.Handler"}))
        .unwrap();
    assert_eq!(hits[0]["id"], "go@1.27/std/net/http::part-2");
    assert_eq!(hits[0]["symbol"], "http.Handler");
    assert_eq!(hits[0]["match_text"], "type Handler interface {");

    let hits = fixture
        .call(json!({"command":"docs.search","query":"go:htt.Handler","pack_id":"python"}))
        .unwrap();
    assert_eq!(hits, json!([]));
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

fn page(id: &str, title: &str, body: &str, html: &str, aliases: &[&str]) -> Value {
    let mut article = doc(id, title, body, aliases);
    article["html"] = json!(html);
    article
}

fn search(fixture: &Fixture, mut request: Value) -> Vec<Value> {
    request["command"] = json!("docs.search");
    fixture.call(request).unwrap().as_array().unwrap().clone()
}

fn titles(hits: &[Value]) -> Vec<&str> {
    hits.iter()
        .map(|hit| hit["title"].as_str().unwrap())
        .collect()
}

#[test]
fn a_bare_name_leads_with_articles_named_for_it_and_docsets_take_turns() {
    // One docset indexes forty members under the bare name. It must not
    // crowd out the types named Map or the other docsets' members.
    let mut java: Vec<Value> = (0..40)
        .map(|n| {
            doc(
                &format!("api/Stream{n:02}"),
                &format!("Stream{n:02}"),
                "map(mapper)\n\nReturns the mapped stream.",
                &["map"],
            )
        })
        .collect();
    java.push(doc(
        "api/Map",
        "Map",
        "Interface Map<K,V>",
        &["java.util.Map"],
    ));
    let fixture = Fixture::with(vec![
        ("java", "Java SE", "java", "26", json!(java)),
        (
            "elixir",
            "Elixir",
            "elixir",
            "1.20",
            json!([
                doc(
                    "Map.html",
                    "Map",
                    "Maps are the key-value data structure.",
                    &["Map"]
                ),
                doc(
                    "Enum.html",
                    "Enum",
                    "map(enumerable, fun)\n\nMaps every element.",
                    &["Enum.map/2"]
                ),
            ]),
        ),
        (
            "ruby",
            "Ruby",
            "ruby",
            "4",
            json!([doc(
                "method/Array#map",
                "Array#map",
                "Array#map returns a new array.",
                &["Array#map"]
            )]),
        ),
    ]);
    let hits = search(&fixture, json!({"query":"map","limit":8}));
    assert_eq!(titles(&hits)[..2], ["Map", "Map"]);
    let leading: Vec<_> = hits[..6]
        .iter()
        .map(|hit| hit["pack_id"].as_str().unwrap())
        .collect();
    for pack in ["java", "elixir", "ruby"] {
        assert!(leading.contains(&pack), "{pack} in {leading:?}");
    }
    assert!(titles(&hits[..6]).contains(&"Enum"));
    assert!(
        hits[..6]
            .iter()
            .filter(|hit| hit["pack_id"] == "java")
            .count()
            <= 3
    );
    // The same query gives the same order.
    assert_eq!(hits, search(&fixture, json!({"query":"map","limit":8})));
}

#[test]
fn names_that_complete_the_typed_word_lead_articles_that_mention_it() {
    let fixture = Fixture::with(vec![
        (
            "python",
            "Python",
            "python",
            "3.15",
            json!([
                doc(
                    "library/pathlib.html",
                    "pathlib — Object-oriented filesystem paths",
                    "Path.read_text(encoding=None)\n\nReturn the decoded contents.",
                    &["pathlib", "pathlib.Path.read_text"]
                ),
                doc(
                    "whatsnew/3.10.html",
                    "What's New In Python 3.10",
                    "pathlib gained features. pathlib pathlib pathlib.",
                    &[]
                ),
                doc(
                    "whatsnew/3.11.html",
                    "What's New In Python 3.11",
                    "pathlib pathlib pathlib changes.",
                    &[]
                ),
                doc(
                    "library/io.html",
                    "io — Core tools for working with streams",
                    "read(size)\n\nRead and return bytes. T is a type variable.",
                    &["io.read"]
                ),
            ]),
        ),
        (
            "ui",
            "UI Kit",
            "swift",
            "1",
            json!([
                doc(
                    "symbol/c",
                    "NavigationSplitViewColumn",
                    "struct NavigationSplitViewColumn",
                    &["NavigationSplitViewColumn"]
                ),
                doc(
                    "symbol/s",
                    "NavigationSplitViewStyle",
                    "protocol NavigationSplitViewStyle",
                    &["NavigationSplitViewStyle"]
                ),
                doc(
                    "symbol/v",
                    "NavigationSplitView",
                    "struct NavigationSplitView",
                    &["NavigationSplitView"]
                ),
            ]),
        ),
    ]);
    let hits = search(&fixture, json!({"query":"pathli"}));
    assert_eq!(hits[0]["id"], "python@3.15/library/pathlib.html");

    // A name in snake case is completed as a name. The single letter after
    // the underscore is not required of every article as a word of its own.
    let hits = search(&fixture, json!({"query":"read_t"}));
    assert_eq!(hits[0]["id"], "python@3.15/library/pathlib.html");
    assert_eq!(hits[0]["symbol"], "pathlib.Path.read_text");
    assert!(hits
        .iter()
        .all(|hit| hit["id"] != "python@3.15/library/io.html"));

    // The shortest completion is the closest one.
    let hits = search(&fixture, json!({"query":"NavigationSp"}));
    assert_eq!(
        titles(&hits),
        [
            "NavigationSplitView",
            "NavigationSplitViewStyle",
            "NavigationSplitViewColumn"
        ]
    );
}

#[test]
fn misspellings_are_corrected_even_when_the_library_repeats_them() {
    let mut notes: Vec<Value> = (0..44)
        .map(|n| {
            doc(
                &format!("guide/note-{n:02}"),
                &format!("Note {n:02}"),
                "The length of a list.\n\nThe order of middleware matters.\n\nSet environment variables first.",
                &[],
            )
        })
        .collect();
    notes.push(doc(
        "api/Array",
        "Array",
        "length\n\nReturns the number of elements.",
        &["Array.length"],
    ));
    notes.push(doc(
        "reference/useState",
        "useState",
        "useState(initialState)",
        &["useState"],
    ));
    let fixture = Fixture::with(vec![
        ("notes", "Notes", "notes", "1", json!(notes)),
        (
            "errata",
            "Errata",
            "errata",
            "1",
            json!([doc(
                "typos",
                "Errata",
                "A page that spells it lenght and enviroment.",
                &[]
            )]),
        ),
    ]);
    // The misspelling exists in the library, but not where the search looks.
    // The name one letter away leads, and the word in running text follows.
    let hits = search(&fixture, json!({"query":"lenght","pack_id":"notes"}));
    assert_eq!(hits[0]["symbol"], "Array.length");
    assert_eq!(hits[1]["match_text"], "The length of a list.");
    // Where it does exist, the literal match stays first and the likely
    // spelling follows it.
    let hits = search(&fixture, json!({"query":"lenght"}));
    assert_eq!(hits[0]["pack_id"], "errata");
    assert!(hits.len() > 1);
    // One wrong word in a longer query.
    let hits = search(
        &fixture,
        json!({"query":"enviroment variables","pack_id":"notes"}),
    );
    assert_eq!(hits[0]["match_text"], "Set environment variables first.");
    // Two slips in one long word.
    let hits = search(&fixture, json!({"query":"middlewear","pack_id":"notes"}));
    assert_eq!(hits[0]["match_text"], "The order of middleware matters.");
    // A transposition in a name resolves to the symbol, bare or qualified.
    let hits = search(&fixture, json!({"query":"useStaet"}));
    assert_eq!(hits[0]["symbol"], "useState");
    let hits = search(&fixture, json!({"query":"Array.lenght"}));
    assert_eq!(hits[0]["symbol"], "Array.length");
}

#[test]
fn spellings_without_case_or_separators_find_the_same_symbol() {
    let fixture = Fixture::new();
    for query in [
        "readtext",
        "read text",
        "READ_TEXT",
        "Read_Text",
        "path read_text",
    ] {
        let hits = search(&fixture, json!({"query":query}));
        assert_eq!(hits[0]["id"], "python@3.15/library/pathlib.html", "{query}");
        assert_eq!(hits[0]["symbol"], "pathlib.Path.read_text", "{query}");
    }
    let hits = search(&fixture, json!({"query":"pathlib path"}));
    assert_eq!(hits[0]["id"], "python@3.15/library/pathlib.html");
    assert_eq!(hits[0]["symbol"], "pathlib.Path");
}

#[test]
fn a_question_is_answered_by_the_section_headed_with_its_words() {
    let fixture = Fixture::with(vec![(
        "python",
        "Python",
        "python",
        "3.15",
        json!([
            page("tutorial/inputoutput.html", "7. Input and output", "Files can be opened.",
                "<h2 id=\"reading-and-writing-files\">Reading and writing files</h2><p>open() returns a file object.</p>", &[]),
            page("library/configparser.html", "configparser — Configuration file parser", "Parses configuration.",
                "<dl><dt id=\"configparser.ConfigParser.read_file\">read_file(f, source=None)</dt></dl>", &[]),
            page("c-api/file.html", "File objects", "C API.",
                "<dl><dt id=\"c.PyFile_ReadLine\">PyObject *PyFile_ReadLine(PyObject *file, int n)</dt></dl>", &[]),
            doc("library/functions.html", "Built-in Functions",
                "abs(x)\n\nReturn the absolute value of a number.\n\nopen(file, mode='r')\n\nOpen file and return a corresponding file object.", &[]),
        ]),
    )]);
    for query in ["read a file", "how do I read a file"] {
        let hits = search(&fixture, json!({"query":query}));
        assert_eq!(
            hits[0]["id"], "python@3.15/tutorial/inputoutput.html",
            "{query}"
        );
        assert_eq!(hits[0]["match_text"], "Reading and writing files");
        assert_eq!(hits[0]["match_anchor"], "reading-and-writing-files");
    }
    // A match in running text shows the line that matched, not the top of the article.
    let hits = search(&fixture, json!({"query":"corresponding object"}));
    assert_eq!(hits[0]["id"], "python@3.15/library/functions.html");
    assert_eq!(
        hits[0]["match_text"],
        "Open file and return a corresponding file object."
    );
    assert!(hits[0]["snippet"]
        .as_str()
        .unwrap()
        .starts_with("Open file and return"));
}

#[test]
fn the_article_about_a_topic_leads_articles_with_a_chapter_on_it() {
    let fixture = Fixture::with(vec![
        (
            "phoenix",
            "Phoenix",
            "elixir",
            "1.8",
            json!([
                page("heroku.html", "Deploying on Heroku", "Heroku steps.",
                    "<h1 id=\"deploying-on-heroku\">Deploying on Heroku</h1><h2 id=\"deploy-time\">Deploy Time!</h2>", &[]),
                page("fly.html", "Deploying on Fly.io", "Fly steps.",
                    "<h1 id=\"deploying-on-fly-io\">Deploying on Fly.io</h1><h2 id=\"deploy-time\">Deploy Time!</h2>", &[]),
                page("deployment.html", "Introduction to Deployment", "An overview.",
                    "<h1 id=\"introduction-to-deployment\">Introduction to Deployment</h1>", &[]),
            ]),
        ),
        (
            "aspnet",
            "ASP.NET Core",
            "dotnet",
            "10",
            json!([
                // The file name makes `middleware` an alias of a narrower article.
                doc("performance/caching/middleware", "Response caching middleware in ASP.NET Core", "Caches responses.", &["middleware"]),
                doc("test/middleware", "Test ASP.NET Core middleware", "Tests a pipeline.", &["middleware"]),
                page("fundamentals/middleware/index", "ASP.NET Core middleware", "The request pipeline.",
                    "<h1 id=\"aspnet-core-middleware\">ASP.NET Core middleware</h1>", &[]),
            ]),
        ),
        (
            "elixir",
            "Elixir",
            "elixir",
            "1.20",
            json!([
                doc("typespecs.html", "Typespecs reference", "def parse(str), do: {:ok, \"some json \" <> str}", &[]),
                doc("JSON.html", "JSON", "Encode and decode data.", &["JSON"]),
            ]),
        ),
    ]);
    // Stems join deploy and deployment, and words such as Introduction say
    // what kind of page it is, not what it is about.
    let hits = search(&fixture, json!({"query":"deploy","pack_id":"phoenix"}));
    assert_eq!(hits[0]["title"], "Introduction to Deployment");
    // The docset's own name in a title is not a second topic.
    let hits = search(&fixture, json!({"query":"middleware","stack":"dotnet"}));
    assert_eq!(hits[0]["title"], "ASP.NET Core middleware");
    assert_eq!(hits.len(), 3);
    // An article named for one of the words is the subject, even when its
    // text says decode where the query says parse.
    let hits = search(&fixture, json!({"query":"parse json","stack":"elixir"}));
    assert_eq!(hits[0]["title"], "JSON");
}

#[test]
fn flags_and_dotted_keys_match_as_written() {
    let fixture = Fixture::with(vec![
        (
            "spring-boot",
            "Spring Boot",
            "java",
            "4.1",
            json!([
                page(
                    "reference/actuator/monitoring.html",
                    "Spring Boot · Monitoring and Management Over HTTP",
                    "Management endpoints.",
                    "<h2 id=\"port\">Customizing the Management Server Port</h2>",
                    &[]
                ),
                doc(
                    "appendix/application-properties.html",
                    "Spring Boot · Common Application Properties",
                    "server.address\n\nserver.port=8080 sets the HTTP port.\n\nserver.shutdown",
                    &[]
                ),
            ]),
        ),
        (
            "playwright",
            "Playwright",
            "node",
            "1.65",
            json!([
                page(
                    "network.html",
                    "Network",
                    "HTTP traffic.",
                    "<h2 id=\"head\">Head</h2>",
                    &[]
                ),
                doc(
                    "running-tests.html",
                    "Running tests",
                    "Tests run headless by default.\n\nSee the browser window: add --headed.",
                    &[]
                ),
            ]),
        ),
    ]);
    let hits = search(&fixture, json!({"query":"server.port"}));
    assert_eq!(
        hits[0]["id"],
        "spring-boot@4.1/appendix/application-properties.html"
    );
    assert_eq!(
        hits[0]["match_text"],
        "server.port=8080 sets the HTTP port."
    );
    let hits = search(&fixture, json!({"query":"--headed"}));
    assert_eq!(hits[0]["id"], "playwright@1.65/running-tests.html");
    assert_eq!(
        hits[0]["match_text"],
        "See the browser window: add --headed."
    );
}

#[test]
fn a_member_is_found_through_its_owner() {
    let fixture = Fixture::with(vec![
        (
            "ruby",
            "Ruby",
            "ruby",
            "4",
            json!([
                doc("doc/box.md", "Ruby Box", "Hash#map calls Hash#each to retrieve entries.", &[]),
                doc("api/Hash", "Hash", "Hash < Object\n\n• #each: Calls the block with each key-value pair.\n\n• #fetch: Returns the value.", &["Hash"]),
            ]),
        ),
        (
            "numpy",
            "NumPy",
            "python",
            "2.5",
            json!([
                doc("reference/generated/numpy.ma.array", "numpy.ma.array", "numpy.ma.array(data)", &["numpy.ma.array"]),
                doc("reference/generated/numpy.array", "numpy.array", "numpy.array(object, dtype=None)", &["numpy.array"]),
            ]),
        ),
    ]);
    // The reference lists the method in its owner's article, not as an alias.
    let hits = search(&fixture, json!({"query":"Hash#each"}));
    assert_eq!(hits[0]["id"], "ruby@4/api/Hash");
    assert_eq!(hits[0]["symbol"], "Hash#each");
    assert_eq!(hits[0]["match_kind"], "symbol");
    // The usual short name of the owner.
    let hits = search(&fixture, json!({"query":"np.array"}));
    assert_eq!(titles(&hits)[..2], ["numpy.array", "numpy.ma.array"]);
    assert_eq!(hits[0]["symbol"], "numpy.array");
}

#[test]
fn browsing_orders_numbered_titles_by_number() {
    let fixture = Fixture::with(vec![(
        "guide",
        "Guide",
        "guide",
        "1",
        json!([
            doc("c10", "10. Last", "x", &[]),
            doc("new310", "What's New In 3.10", "x", &[]),
            doc("c2", "2. Second", "x", &[]),
            doc("appendix", "<em>Appendix</em>", "x", &[]),
            doc("new39", "What's New In 3.9", "x", &[]),
            doc("c1", "1. First", "x", &[]),
        ]),
    )]);
    let all = search(&fixture, json!({"query":"","pack_id":"guide"}));
    assert_eq!(
        titles(&all),
        [
            "1. First",
            "2. Second",
            "10. Last",
            "Appendix",
            "What's New In 3.9",
            "What's New In 3.10"
        ]
    );
    let page = search(
        &fixture,
        json!({"query":"","pack_id":"guide","limit":2,"offset":2}),
    );
    assert_eq!(titles(&page), ["10. Last", "Appendix"]);
}
