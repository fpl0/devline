use devkit_core::execute;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
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

    fn committed_image_update(&self) -> PathBuf {
        let architecture = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "amd64"
        };
        let platform = format!("linux/{architecture}");
        let catalog: Value =
            serde_json::from_slice(include_bytes!("../../data/infrastructure.json")).unwrap();
        let mut service = catalog["services"]
            .as_array()
            .unwrap()
            .iter()
            .find(|service| service["id"] == "postgres" && service["image"]["platform"] == platform)
            .unwrap()
            .clone();
        fs::write(
            self.0.join("catalog/infrastructure.json"),
            json!({"schema_version":1,"services":[service.clone()]}).to_string(),
        )
        .unwrap();
        service["version"] = json!("18.7");
        service["image"]["tag"] = json!("18.7-alpine");
        let config =
            json!({"os":"linux","architecture":architecture,"rootfs":{"diff_ids":[]}}).to_string();
        service["image"]["config_digest"] =
            json!(format!("sha256:{:x}", Sha256::digest(config.as_bytes())));
        let manifest = br#"[{"Config":"config.json","Layers":[],"RepoTags":null}]"#;
        let compressed = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut archive = tar::Builder::new(compressed);
        for (name, bytes) in [
            ("config.json", config.as_bytes()),
            ("manifest.json", manifest.as_slice()),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(bytes.len() as u64);
            header.set_cksum();
            archive.append_data(&mut header, name, bytes).unwrap();
        }
        let bytes = archive.into_inner().unwrap().finish().unwrap();
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let root = self.0.join("library/infrastructure");
        fs::create_dir_all(root.join("images")).unwrap();
        let path = root.join(format!("images/{hash}.tar.gz"));
        fs::write(&path, &bytes).unwrap();
        let image = &service["image"];
        let receipt = json!({"id":"postgres","version":"18.7","platform":platform,
            "source":format!("{}/{}@{}",image["registry"].as_str().unwrap(),image["repository"].as_str().unwrap(),image["digest"].as_str().unwrap()),
            "source_digest":image["digest"],"image_id":image["config_digest"],
            "archive_sha256":hash,"size_bytes":bytes.len(),"verified_at":1});
        fs::write(root.join("index.json"), json!({format!("postgres@18.7/{platform}"):{
            "service":service,"receipt":receipt,"stamp":{"size":bytes.len(),"seconds":0,"nanos":0,"inode":0}
        }}).to_string()).unwrap();
        fs::write(root.join("updates.json"), json!({format!("postgres/{platform}"):{"checked_at":1,"error":null,"offer":{
            "version":"18.7","tag":"18.7-alpine","index_digest":image["index_digest"],"digest":image["digest"],
            "config_digest":image["config_digest"],"compressed_bytes":bytes.len()
        }}}).to_string()).unwrap();
        // Simulate cancellation after the archive commit, before the aggregate
        // update could clear its durable pending record.
        fs::write(self.0.join("library/content-update-pending.json"), json!({"schema_version":1,"items":[{
            "id":"image:postgres","name":"PostgreSQL","kind":"image","state":"available",
            "action":"update_image","reference":"postgres@18.7","current_version":"18.6","available_version":"18.7"
        }]}).to_string()).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn committed_image_updates_retry_by_verifying_the_exact_local_archive() {
    for offline in [false, true] {
        let fixture = Fixture::new();
        let archive = fixture.committed_image_update();
        let original = fs::read(&archive).unwrap();
        let direct = fixture.call(json!({"command":"infra.update","id":"postgres@18.7"}));
        assert!(direct
            .unwrap_err()
            .to_string()
            .contains("No newer compatible"));
        let result = fixture
            .call(json!({"command":"updates.apply","id":"postgres@18.7","offline":offline}))
            .unwrap();
        assert_eq!(result["complete"], true, "{result}");
        assert_eq!(result["updated"], 1);
        assert_eq!(result["steps"][0]["reference"], "postgres@18.7");
        assert!(Fixture::item(&result["snapshot"], "image:postgres")["pending"].is_null());
        assert_eq!(fs::read(&archive).unwrap(), original);
    }
}

#[test]
fn missing_or_corrupt_retry_archives_remain_pending_until_verified() {
    let fixture = Fixture::new();
    let archive = fixture.committed_image_update();
    let original = fs::read(&archive).unwrap();
    let mut corrupt = original.clone();
    corrupt[0] ^= 1;
    fs::write(&archive, corrupt).unwrap();
    for missing in [false, true] {
        if missing {
            fs::remove_file(&archive).unwrap();
        }
        let result = fixture
            .call(json!({"command":"updates.apply","id":"image:postgres","offline":true}))
            .unwrap();
        assert_eq!(result["complete"], false, "{result}");
        assert_eq!(result["failed"], 1);
        let pending = Fixture::item(&result["snapshot"], "image:postgres");
        assert_eq!(pending["pending"], true);
        assert_eq!(pending["reference"], "postgres@18.7");
    }
    fs::write(&archive, original).unwrap();
    let recovered = fixture
        .call(json!({"command":"updates.apply","id":"image:postgres","offline":true}))
        .unwrap();
    assert_eq!(recovered["complete"], true, "{recovered}");
    assert!(Fixture::item(&recovered["snapshot"], "image:postgres")["pending"].is_null());
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

#[test]
fn bulk_offline_update_installs_bundled_docs_without_claiming_unchecked_sources_current() {
    let fixture = Fixture::new();
    // Restrict this fixture to a documentation library: no external build tools
    // or network are involved in applying a bundled reference update.
    let path = fixture.0.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    catalog["templates"] = json!([]);
    fs::write(&path, catalog.to_string()).unwrap();
    fixture
        .call(json!({"command":"docs.install","id":"go","version":"1.27.2"}))
        .unwrap();
    let result = fixture
        .call(json!({"command":"updates.apply","offline":true}))
        .unwrap();
    assert_eq!(result["updated"], 1);
    assert_eq!(result["complete"], false);
    assert!(result["failed"].as_u64().unwrap() > 0);
    assert!(result["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step["id"] == "docs:go" && step["state"] == "succeeded"));
    let docs = fixture
        .call(json!({"command":"docs.distribution.status"}))
        .unwrap();
    let versions = &docs["packs"][0]["installed_versions"];
    assert!(versions.as_array().unwrap().contains(&json!("1.27.2")));
    assert!(versions.as_array().unwrap().contains(&json!("1.27.3")));
    let result = fixture.call(json!({"command":"updates.check","offline":true}));
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires a connection"));
}

#[test]
fn app_only_status_preserves_independent_source_freshness_and_rejects_offline_fetch() {
    let fixture = Fixture::new();
    fixture
        .call(json!({"command":"updates.configure","url":"https://example.org/release.json"}))
        .unwrap();
    let settings = fixture.0.join("library/updates.json");
    fs::write(&settings, json!({
        "app_feed":"https://example.org/release.json","checked_at":123,"app_checked_at":456,
        "observations":{"app":{"version":"9.0.0","compatible":null,"url":"https://example.org/releases/9.0.0","error":null}}
    }).to_string()).unwrap();
    let app = fixture
        .call(json!({"command":"updates.app.status"}))
        .unwrap();
    assert_eq!(app["checked_at"], 456);
    assert_eq!(app["items"].as_array().unwrap().len(), 1);
    assert_eq!(app["items"][0]["available_version"], "9.0.0");
    let all = fixture.call(json!({"command":"updates.status"})).unwrap();
    assert!(all["checked_at"].is_null());
    assert_eq!(all["all_checked"], false);
    let upstream = all["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|source| source["id"] == "upstream")
        .unwrap();
    assert_eq!(upstream["checked_at"], 123);
    let before = fs::read(&settings).unwrap();
    assert!(fixture
        .call(json!({"command":"updates.app.check","offline":true}))
        .is_err());
    assert_eq!(fs::read(&settings).unwrap(), before);
    fixture
        .call(json!({"command":"updates.configure","url":""}))
        .unwrap();
    let app = fixture
        .call(json!({"command":"updates.app.check"}))
        .unwrap();
    assert_eq!(app["items"][0]["state"], "unconfigured");
    let saved: Value = serde_json::from_slice(&fs::read(&settings).unwrap()).unwrap();
    assert_eq!(saved["checked_at"], 123);
    assert!(saved["app_checked_at"].is_null());
}

#[cfg(target_os = "macos")]
#[test]
fn a_failed_installed_starter_update_remains_retryable_until_offline_verification_passes() {
    let fixture = Fixture::new();
    let manifest = fixture.0.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    let repaired = fixture.0.join("verification-can-succeed");
    for template in catalog["templates"].as_array_mut().unwrap() {
        template["toolchain"] = json!("true");
        template["minimum_version"] = json!("");
        template["test_command"] = json!("./test-offline.sh");
        template["docs"] = json!([]);
        let path = fixture
            .0
            .join("catalog")
            .join(template["path"].as_str().unwrap())
            .join("project/test-offline.sh");
        fs::write(
            path,
            format!("#!/bin/sh\nset -eu\ntest -f '{}'\n", repaired.display()),
        )
        .unwrap();
    }
    fs::write(&manifest, catalog.to_string()).unwrap();
    fixture
        .call(json!({"command":"templates.install","id":"go/chi-api@1.0.0"}))
        .unwrap();
    let first = fixture
        .call(json!({"command":"updates.apply","id":"go/chi-api@1.1.0","offline":true}))
        .unwrap();
    assert_eq!(first["complete"], false);
    assert_eq!(first["failed"], 1);
    assert!(fixture
        .0
        .join("library/templates/go/chi-api/1.1.0")
        .is_dir());
    let snapshot = fixture.call(json!({"command":"updates.status"})).unwrap();
    let pending = Fixture::item(&snapshot, "template:go/chi-api");
    assert_eq!(pending["pending"], true);
    assert_eq!(pending["reference"], "go/chi-api@1.1.0");
    assert_eq!(pending["action"], "install_template");
    let repeated = fixture
        .call(json!({"command":"updates.apply","id":"template:go/chi-api","offline":true}))
        .unwrap();
    assert_eq!(repeated["complete"], false);
    assert_eq!(repeated["failed"], 1);
    // Repair an external requirement without changing the installed release.
    fs::write(repaired, "ready").unwrap();
    let finished = fixture
        .call(json!({"command":"updates.apply","id":"template:go/chi-api","offline":true}))
        .unwrap();
    assert_eq!(finished["complete"], true);
    assert_eq!(finished["updated"], 1);
    let doctor = fixture
        .call(json!({"command":"doctor","id":"go/chi-api@1.1.0"}))
        .unwrap();
    assert_eq!(doctor["ready_offline"], true);
    let snapshot = fixture.call(json!({"command":"updates.status"})).unwrap();
    assert!(Fixture::item(&snapshot, "template:go/chi-api")["pending"].is_null());
    assert!(fixture
        .0
        .join("library/templates/go/chi-api/1.0.0")
        .is_dir());
}
