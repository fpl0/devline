use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let fixture = Self(std::env::temp_dir().join(format!(
            "devline-project-{}-{unique}-{sequence}",
            std::process::id()
        )));
        let mut packs = Vec::new();
        for (id, stack, version) in [
            ("python", "python", "3.12.9"),
            ("python", "python", "3.15.2"),
            ("fastapi", "python", "0.120.0"),
            ("fastapi", "python", "0.162.0+devline.1"),
            ("go", "go", "1.27.3"),
            ("csharp", "dotnet", "2026.11.06+devline.4"),
            ("aspnet", "dotnet", "2026.11.06+devline.3"),
            ("swift", "swift", "6.3.0"),
            ("swift", "swift", "6.4.0"),
            ("tauri", "rust", "2.12.5"),
        ] {
            let path = format!("docs/{id}/{version}");
            let document = json!([{"id":"guide","title":id,"body":"Compatibility sample", "source_url":"https://example.org/"}]);
            fixture.write(
                &format!("catalog/{path}/documents.json"),
                &document.to_string(),
            );
            let mut pack = json!({"id":id,"name":id,"stack":stack,"version":version,"description":"Fixture","source_url":"https://example.org/","license":"MIT","path":path});
            if stack == "dotnet" {
                pack["upstream_version"] = json!("10.0");
            }
            packs.push(pack);
        }
        fixture.write(
            "catalog/catalog.json",
            &json!({"schema_version":1,"docs":packs,"templates":[]}).to_string(),
        );
        for pack in packs {
            fixture
                .call(json!({"command":"docs.install","id":pack["id"],"version":pack["version"]}));
        }
        fixture
    }

    fn write(&self, name: &str, contents: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn call(&self, mut request: Value) -> Value {
        request["home"] = json!(self.0.join("library"));
        request["catalog"] = json!(self.0.join("catalog"));
        devkit_core::execute(&request).unwrap()
    }

    fn profile(&self, path: &str) -> Value {
        self.call(json!({"command":"project.context","path":self.0.join(path)}))
    }

    fn cli(&self, current: &Path, args: &[&str]) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_devline"))
            .current_dir(current)
            .arg("--home")
            .arg(self.0.join("library"))
            .arg("--catalog")
            .arg(self.0.join("catalog"))
            .arg("--json")
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn entry<'a>(profile: &'a Value, list: &str, id: &str) -> &'a Value {
    profile[list]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
}

#[test]
fn swift_package_tools_version_is_a_minimum_and_detects_swiftui_sources() {
    let fixture = Fixture::new();
    fixture.write(
        "desktop/Package.swift",
        "// swift-tools-version: 6.3\nimport PackageDescription\nfatalError(\"do not evaluate manifests\")\n",
    );
    fixture.write(
        "desktop/Sources/ReadingDesk/Views/ContentView.swift",
        "@preconcurrency import struct SwiftUI.Text\n",
    );
    let profile = fixture.profile("desktop/Sources/ReadingDesk/Views/ContentView.swift");
    assert_eq!(profile["stacks"], json!(["swift"]));
    assert_eq!(
        entry(&profile, "dependencies", "swift")["requirement"],
        ">=6.3"
    );
    assert_eq!(entry(&profile, "docsets", "swift")["version"], "6.4.0");
    assert_eq!(
        entry(&profile, "dependencies", "swiftui")["source"],
        "Sources"
    );
    assert!(entry(&profile, "docsets", "swiftui")["version"].is_null());

    fixture.write("desktop/.swift-version", "6.3.2\n");
    let profile = fixture.profile("desktop");
    assert_eq!(
        entry(&profile, "dependencies", "swift")["source"],
        ".swift-version"
    );
    assert_eq!(entry(&profile, "docsets", "swift")["version"], "6.3.0");
}

#[test]
fn swiftui_detection_ignores_comments_strings_tests_and_symlinked_sources() {
    let fixture = Fixture::new();
    fixture.write("library/.swift-version", "6.4.2\n");
    fixture.write(
        "library/Sources/Example.swift",
        "// import SwiftUI\n/*\n  /* nested */\n  import SwiftUI\n*/\nlet sample = \"\"\"\nimport SwiftUI\n\"\"\"\n",
    );
    fixture.write("library/Tests/UITests.swift", "import SwiftUI\n");
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        fixture.0.join("library/Tests/UITests.swift"),
        fixture.0.join("library/Sources/Linked.swift"),
    )
    .unwrap();
    let profile = fixture.profile("library");
    assert_eq!(profile["stacks"], json!(["swift"]));
    assert!(profile["dependencies"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry["id"] != "swiftui"));
}

#[test]
fn rust_project_resolves_renamed_tauri_workspace_dependency_from_lockfile() {
    let fixture = Fixture::new();
    fixture.write(
        "workspace/Cargo.toml",
        "[workspace]\nmembers=['desktop']\n[workspace.dependencies]\ndesktop-framework={package='tauri',version='2.12'}\n",
    );
    fixture.write(
        "workspace/desktop/Cargo.toml",
        "[package]\nname='desktop'\n[dependencies]\ndesktop-framework.workspace=true\naxum='0.8'\n",
    );
    fixture.write(
        "workspace/Cargo.lock",
        "[[package]]\nname='tauri'\nversion='1.8.0'\n[[package]]\nname='tauri'\nversion='2.12.5'\n[[package]]\nname='axum'\nversion='0.8.9'\n",
    );
    let profile = fixture.profile("workspace/desktop");
    assert_eq!(profile["stacks"], json!(["rust"]));
    let tauri = entry(&profile, "dependencies", "tauri");
    assert_eq!(tauri["source"], "Cargo.lock");
    assert_eq!(tauri["requirement"], "2.12");
    assert_eq!(tauri["version"], "2.12.5");
    assert_eq!(entry(&profile, "dependencies", "axum")["version"], "0.8.9");
    assert_eq!(entry(&profile, "docsets", "tauri")["version"], "2.12.5");
}

#[test]
fn project_search_uses_lock_versions_and_keeps_intentional_incompatible_pins() {
    let fixture = Fixture::new();
    fixture.write("service/pyproject.toml", "[project]\nname='service'\nrequires-python='>=3.12,<3.14'\ndependencies=['fastapi[standard]>=0.115']\n");
    fixture.write(
        "service/uv.lock",
        "[[package]]\nname='fastapi'\nversion='0.120.0'\n",
    );
    fixture.write("service/api/routes.py", "");
    let root = fixture.0.join("service").canonicalize().unwrap();
    let profile = fixture.profile("service/api/routes.py");
    assert_eq!(profile["path"], json!(root));
    assert_eq!(
        entry(&profile, "dependencies", "fastapi")["source"],
        "uv.lock"
    );
    assert_eq!(entry(&profile, "docsets", "python")["version"], "3.12.9");
    let search = fixture.cli(&root, &["docs", "search", "Compatibility sample"]);
    let releases: Vec<_> = search
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| hit["version"].as_str().unwrap())
        .collect();
    assert_eq!(releases.len(), 2);
    assert!(releases.contains(&"3.12.9") && releases.contains(&"0.120.0"));
    let exact = fixture.cli(&root, &["docs", "show", "fastapi@0.162.0+devline.1/guide"]);
    assert_eq!(exact["version"], "0.162.0+devline.1");
    let override_scope = fixture.cli(
        &root,
        &["docs", "search", "fastapi@0.162.0+devline.1:Compatibility"],
    );
    assert_eq!(override_scope[0]["version"], "0.162.0+devline.1");

    fixture.write(
        "service/uv.lock",
        "[[package]]\nname='fastapi'\nversion='0.118.0'\n",
    );
    let profile = fixture.profile("service/api");
    assert_eq!(entry(&profile, "docsets", "fastapi")["state"], "unmatched");
    assert!(entry(&profile, "docsets", "fastapi")["version"].is_null());
    let search = fixture.cli(&root, &["docs", "search", "Compatibility"]);
    assert_eq!(search.as_array().unwrap().len(), 1);
    assert_eq!(search[0]["pack_id"], "python");
    let pinned = fixture.cli(
        &root.join("api"),
        &["project", "pin", "fastapi@0.162.0+devline.1"],
    );
    assert_eq!(entry(&pinned, "docsets", "fastapi")["state"], "pinned");
    assert_eq!(entry(&pinned, "docsets", "fastapi")["compatible"], false);
    let search = fixture.cli(&root, &["docs", "search", "Compatibility"]);
    assert!(search
        .as_array()
        .unwrap()
        .iter()
        .any(|hit| hit["version"] == "0.162.0+devline.1"));
    let unpinned = fixture.cli(&root, &["project", "unpin", "fastapi"]);
    assert_eq!(entry(&unpinned, "docsets", "fastapi")["state"], "unmatched");
}

#[test]
fn detects_the_other_six_stacks_and_resolved_framework_versions() {
    let fixture = Fixture::new();
    fixture.write(
        "go/go.mod",
        "module example.org/service\ngo 1.27\nrequire (\n github.com/go-chi/chi/v5 v5.2.4\n)\n",
    );
    fixture.write(
        "rust/Cargo.toml",
        "[package]\nname='sample'\n[dependencies]\nrouter={package='axum',version='0.8'}\n",
    );
    fixture.write(
        "rust/rust-toolchain.toml",
        "[toolchain]\nchannel='1.99.1'\n",
    );
    fixture.write(
        "rust/Cargo.lock",
        "[[package]]\nname='axum'\nversion='0.7.9'\n[[package]]\nname='axum'\nversion='0.8.9'\n",
    );
    fixture.write("ruby/.ruby-version", "4.0.9\n");
    fixture.write("ruby/Gemfile", "gem 'rails', '~> 8.1'\n");
    fixture.write("ruby/Gemfile.lock", "GEM\n  specs:\n    rails (8.1.3)\n");
    fixture.write(
        "java/pom.xml",
        r#"<project xmlns="http://maven.apache.org/POM/4.0.0">
      <parent><groupId>org.springframework.boot</groupId><version>4.1.2</version></parent>
      <properties><java.version>26</java.version></properties></project>"#,
    );
    fixture.write(
        "elixir/mix.exs",
        r#"def project, do: [elixir: "~> 1.20", deps: [{:phoenix, "~> 1.8"}]]"#,
    );
    fixture.write(
        "elixir/mix.lock",
        r#"%{
      "phoenix": {:hex, :phoenix, "1.8.16", "digest", [:mix], []},
      "ecto": {:hex, :ecto, "3.14.3", "digest", [:mix], []}
    }"#,
    );
    fixture.write("dotnet/Api.csproj", r#"<Project Sdk="Microsoft.NET.Sdk.Web"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>"#);
    for (directory, pack, version, source) in [
        ("go", "chi", "5.2.4", "go.mod"),
        ("rust", "axum", "0.8.9", "Cargo.lock"),
        ("ruby", "rails", "8.1.3", "Gemfile.lock"),
        ("java", "spring-boot", "4.1.2", "pom.xml"),
        ("elixir", "phoenix", "1.8.16", "mix.lock"),
        ("elixir", "ecto", "3.14.3", "mix.lock"),
        ("dotnet", "aspnet", "10.0", "Api.csproj"),
    ] {
        let profile = fixture.profile(directory);
        assert_eq!(profile["stack"], directory);
        let dependency = entry(&profile, "dependencies", pack);
        assert_eq!(dependency["version"], version, "{pack}");
        assert_eq!(dependency["source"], source, "{pack}");
    }
    let profile = fixture.profile("dotnet");
    assert_eq!(entry(&profile, "docsets", "aspnet")["state"], "matched");
    assert_eq!(
        entry(&profile, "docsets", "csharp")["version"],
        "2026.11.06+devline.4"
    );
    fixture.write("dotnet/Api.csproj", r#"<Project Sdk="Microsoft.NET.Sdk.Web"><PropertyGroup><TargetFrameworks>net8.0;net10.0</TargetFrameworks></PropertyGroup></Project>"#);
    assert_eq!(
        entry(&fixture.profile("dotnet"), "docsets", "aspnet")["state"],
        "unmatched"
    );
}

#[test]
fn compatible_project_selection_takes_priority_over_global_rollback() {
    let fixture = Fixture::new();
    fixture.write(
        "service/pyproject.toml",
        "[project]\nname='service'\ndependencies=['fastapi==0.162.0']\n",
    );
    fixture
        .call(json!({"command":"docs.distribution.rollback","id":"fastapi","version":"0.120.0"}));
    let global = fixture.call(json!({"command":"docs.search","query":"fastapi:Compatibility"}));
    assert_eq!(global[0]["version"], "0.120.0");
    assert_eq!(
        entry(&fixture.profile("service"), "docsets", "fastapi")["version"],
        "0.162.0+devline.1"
    );
}

#[test]
fn unavailable_swiftui_reference_remains_an_offline_coverage_gap() {
    let fixture = Fixture::new();
    fixture.write("app/Package.swift", "// swift-tools-version: 6.0\n");
    fixture.write("app/Sources/App.swift", "import SwiftUI\n");
    let profile = fixture.profile("app");
    let gap = entry(&profile, "docsets", "swiftui");
    assert_eq!(gap["state"], "uninstalled");
    assert!(gap["version"].is_null());
    assert!(gap["detail"]
        .as_str()
        .unwrap()
        .contains("Prepare Apple’s SwiftUI API reference from the selected local Xcode SDK"));
    let offline = fixture.call(json!({"command":"offline.status","path":fixture.0.join("app")}));
    assert_eq!(offline["ready_offline"], false);
    assert_eq!(entry(&offline, "docs", "swiftui")["ready_offline"], false);
}

#[test]
fn node_detects_exact_lock_versions_and_runtime_source() {
    let fixture = Fixture::new();
    fixture.write("web/package.json", r#"{"engines":{"node":">=26 <27"},"dependencies":{"react":"^19.0.0"},"devDependencies":{"@playwright/test":"^1.58.0","vite":"^8.0.0"}}"#);
    fixture.write("web/package-lock.json", r#"{"lockfileVersion":3,"packages":{"node_modules/react":{"version":"19.3.1"},"node_modules/@playwright/test":{"version":"1.62.0"},"node_modules/vite":{"version":"8.3.6"}}}"#);
    fixture.write("web/src/main.ts", "");
    let initial = fixture.profile("web/src/main.ts");
    assert_eq!(initial["stack"], "node");
    assert_eq!(
        entry(&initial, "dependencies", "node")["requirement"],
        ">=26 <27"
    );
    assert_eq!(
        entry(&initial, "dependencies", "react")["version"],
        "19.3.1"
    );
    assert_eq!(
        entry(&initial, "dependencies", "playwright")["version"],
        "1.62.0"
    );
    fixture.write("web/.nvmrc", "v26.13.0\n");
    let pinned = fixture.profile("web");
    assert_eq!(entry(&pinned, "dependencies", "node")["source"], ".nvmrc");
    assert_eq!(entry(&pinned, "dependencies", "node")["version"], "26.13.0");
}

#[test]
fn toolbox_prepares_compatible_missing_docs_and_keeps_partial_failures_truthful() {
    let fixture = Fixture::new();
    fixture.write(
        "service/pyproject.toml",
        "[project]\nname='service'\ndependencies=['fastapi==0.120.0']\n",
    );
    fixture.call(json!({"command":"docs.remove","id":"fastapi","version":"0.120.0"}));
    let before = fixture.profile("service");
    assert!(entry(&before, "docsets", "fastapi")["version"].is_null());
    assert_eq!(
        entry(&before, "docsets", "fastapi")["prepare_version"],
        "0.120.0"
    );
    let report = fixture
        .call(json!({"command":"toolbox.prepare","path":fixture.0.join("service"),"offline":true}));
    assert_eq!(
        entry(&report["library"], "docs", "fastapi")["ready_offline"],
        true
    );
    assert_eq!(
        entry(&fixture.profile("service"), "docsets", "fastapi")["version"],
        "0.120.0"
    );
    // This fixture deliberately has no dependency adapter. Docs succeed while the
    // complete project remains honestly unready, with the actionable error kept.
    assert_eq!(report["ready_offline"], false);
    assert!(report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step["kind"] == "project" && step["status"] == "blocked"));
    assert!(report["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step["kind"] == "project"));
}

#[test]
fn toolbox_never_substitutes_an_unavailable_exact_project_pin() {
    let fixture = Fixture::new();
    fixture.write(
        "service/pyproject.toml",
        "[project]\nname='service'\ndependencies=['fastapi==0.120.0']\n",
    );
    let path = fixture.0.join("service");
    fixture.call(json!({"command":"project.pin","path":path,"id":"fastapi","version":"0.119.99"}));
    let report = fixture.call(json!({"command":"toolbox.prepare","path":path,"offline":true}));
    assert_eq!(report["ready_offline"], false);
    let after = fixture.profile("service");
    assert_eq!(entry(&after, "docsets", "fastapi")["version"], "0.119.99");
    assert_eq!(entry(&after, "pins", "fastapi")["version"], "0.119.99");
    assert!(report["steps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|step| step["id"] == "fastapi@0.119.99" && step["status"] == "blocked"));
}

#[test]
fn a_project_origin_is_provenance_but_explicit_starter_selection_is_required() {
    let fixture = Fixture::new();
    fixture.write("service/go.mod", "module example.com/service\ngo 1.27\n");
    fixture.write(
        "service/.devkit-origin.json",
        &json!({"template":"go/removed-starter","version":"1.0.0","digest":"a".repeat(64)})
            .to_string(),
    );
    let path = fixture.0.join("service");
    let inferred = fixture.call(json!({"command":"offline.status","path":path}));
    assert_eq!(
        inferred["project"]["origin"]["reference"],
        "go/removed-starter@1.0.0"
    );
    assert!(inferred["starters"].as_array().unwrap().is_empty());
    let explicit = fixture.call(
        json!({"command":"offline.status","path":path,"starters":["go/removed-starter@1.0.0"]}),
    );
    assert_eq!(explicit["ready_offline"], false);
    assert_eq!(explicit["starters"][0]["ready_offline"], false);
}

#[cfg(target_os = "macos")]
#[test]
fn new_project_uses_the_organized_layout_and_one_launcher() {
    let fixture = Fixture::new();
    let catalog_path = fixture.0.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    catalog["templates"] = json!([{"id":"python/fixture","name":"Fixture","stack":"python","version":"1.0.0","description":"Fixture","app_type":"cli","path":"templates/fixture","toolchain":"python3","minimum_version":"","test_command":"./test-offline.sh","docs":[],"target":"any","verified":true}]);
    fs::write(catalog_path, catalog.to_string()).unwrap();
    let adapter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/project-adapters/python.sh");
    fixture.write(
        "catalog/project-adapters/python.sh",
        &fs::read_to_string(adapter).unwrap(),
    );
    let release = "catalog/templates/fixture/project";
    fixture.write(&format!("{release}/requirements.txt"), "");
    fixture.write(&format!("{release}/wheelhouse/seed"), "bundled");
    let test_script = "#!/bin/sh\nset -eu\ncd \"$(dirname \"$0\")\"\ntest -f requirements.txt\nprintf '%s\\n' \"${1:-tested}\"\n";
    fixture.write(&format!("{release}/test-offline.sh"), test_script);
    fixture.write(
        &format!("{release}/run.sh"),
        "#!/bin/sh\nset -eu\ncd \"$(dirname \"$0\")\"\ntest -f requirements.txt\nprintf '%s\\n' launched\n",
    );
    let run = fixture.0.join(format!("{release}/run.sh"));
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(run, fs::Permissions::from_mode(0o755)).unwrap();
    fixture.write(
        &format!("{release}/devline.json"),
        r#"{"schema_version":1,"run":{"command":["./run.sh"]}}"#,
    );
    let path = fixture.0.join("new-app");
    let plan = fixture.call(
        json!({"command":"new.ready","id":"python/fixture","destination":path,"offline":true,"dry_run":true}),
    );
    assert_eq!(plan["test_command"], "./devline.sh test");
    assert!(!path.exists());
    let created = fixture.call(
        json!({"command":"new.ready","id":"python/fixture","destination":path,"offline":true}),
    );
    assert_eq!(created["test_command"], plan["test_command"]);
    assert_eq!(
        fs::read_dir(&path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<std::collections::BTreeSet<_>>(),
        [".devline", "devline.sh", "requirements.txt"]
            .map(str::to_owned)
            .into()
    );
    assert!(path.join(".devline/packages/wheelhouse/seed").is_file());
    assert_eq!(
        fs::read_to_string(fixture.0.join(format!("{release}/test-offline.sh"))).unwrap(),
        test_script
    );
    let commands = fixture.call(json!({"command":"project.commands","path":path}));
    assert_eq!(commands["launchable"], true);
    assert!(commands["run_command"]
        .as_str()
        .unwrap()
        .ends_with("/devline.sh' run"));
    assert!(commands["test_command"]
        .as_str()
        .unwrap()
        .ends_with("/devline.sh' test"));
    let launcher = |args: &[&str]| {
        let result = Command::new(path.join("devline.sh"))
            .args(args)
            .current_dir(&fixture.0)
            .env("DEVLINE_BIN", env!("CARGO_BIN_EXE_devline"))
            .env("DEVKIT_HOME", fixture.0.join("library"))
            .env("DEVKIT_CATALOG", fixture.0.join("catalog"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).unwrap()
    };
    assert_eq!(launcher(&["test", "two words"]), "two words\n");
    assert_eq!(launcher(&["run"]), "launched\n");
    assert_eq!(
        fixture.call(json!({"command":"project.readiness","path":path}))["ready_offline"],
        true
    );
    fixture.write("new-app/source.py", "print('a source edit')\n");
    assert_eq!(
        fixture.call(json!({"command":"project.readiness","path":path}))["ready_offline"],
        false
    );
    assert_eq!(
        fixture.call(json!({"command":"project.prepare","path":path,"offline":true}))
            ["ready_offline"],
        true
    );
}

#[cfg(target_os = "macos")]
#[test]
fn edited_generated_project_reverifies_local_dependencies_before_any_online_adapter() {
    let fixture = Fixture::new();
    let catalog_path = fixture.0.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    catalog["templates"] = json!([{"id":"python/fixture","name":"Fixture","stack":"python","version":"1.0.0","description":"Fixture","app_type":"cli","path":"templates/fixture","toolchain":"python3","minimum_version":"","test_command":"./test-offline.sh","docs":[],"target":"any"}]);
    fs::write(catalog_path, catalog.to_string()).unwrap();
    // A call to online preparation is a hard failure: the project's closure is
    // already present, so editing its source must only require local verification.
    fixture.write(
        "catalog/project-adapters/python.sh",
        "#!/bin/sh\necho 'unexpected online adapter' >&2\nexit 97\n",
    );
    fixture.write("app/requirements.txt", "");
    fixture.write("app/source.txt", "first");
    fixture.write(
        "app/.devkit-origin.json",
        &json!({"template":"python/fixture","version":"1.0.0","digest":"a".repeat(64)}).to_string(),
    );
    fixture.write(
        "app/test-offline.sh",
        "#!/bin/sh\nset -eu\ntest -f requirements.txt\ntest -s source.txt\n\"$DEVKIT_TOOLCHAIN\" -c 'import errno,socket; s=socket.socket(); s.settimeout(2); assert s.connect_ex((\"1.1.1.1\",443)) in (errno.EPERM,errno.EACCES)'\n",
    );
    let path = fixture.0.join("app");
    let first = fixture.call(json!({"command":"toolbox.prepare","path":path,"offline":true}));
    assert_eq!(first["ready_offline"], true, "{first}");
    fixture.write("app/source.txt", "edited");
    let stale = fixture.call(json!({"command":"toolbox.status","path":path}));
    assert_eq!(stale["ready_offline"], false);
    let output = Command::new(env!("CARGO_BIN_EXE_devline"))
        .arg("--home")
        .arg(fixture.0.join("library"))
        .arg("--catalog")
        .arg(fixture.0.join("catalog"))
        .args(["--json", "toolbox", "prepare", "--path"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["data"]["ready_offline"], true);
    assert_eq!(
        fs::read_to_string(path.join("source.txt")).unwrap(),
        "edited"
    );
    assert!(report["data"]["library"]["starters"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[cfg(unix)]
#[test]
fn project_runtime_constraints_are_independent_of_starter_minimums() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let catalog_path = fixture.0.join("catalog/catalog.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    catalog["templates"] = json!([{"id":"python/fixture","name":"Fixture","stack":"python","version":"1.0.0","description":"Fixture","app_type":"cli","path":"templates/fixture","toolchain":"python3","minimum_version":"3.15","test_command":"./test-offline.sh","docs":[],"target":"any"}]);
    fs::write(catalog_path, catalog.to_string()).unwrap();
    fixture.write(
        "service/pyproject.toml",
        "[project]\nname='service'\nrequires-python='>=3.12,<3.13'\n",
    );
    let tool = fixture.0.join("python3");
    let invoke = |arguments: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_devline"))
            .current_dir(fixture.0.join("service"))
            .env("DEVKIT_PYTHON3", &tool)
            .arg("--home")
            .arg(fixture.0.join("library"))
            .arg("--catalog")
            .arg(fixture.0.join("catalog"))
            .arg("--json")
            .args(arguments)
            .output()
            .unwrap();
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
    };
    fixture.write("python3", "#!/bin/sh\nprintf '3.12.8 cpython arm64\\n'\n");
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
    let report = invoke(&["toolbox", "status"]);
    assert_eq!(report["library"]["runtimes"][0]["version"], "3.12.8");
    assert_eq!(report["library"]["runtimes"][0]["ready_offline"], true);
    let starter = invoke(&["runtimes"]);
    assert_eq!(
        starter[0]["available"], false,
        "starter wheels must retain their own ABI constraints"
    );

    fixture.write("python3", "#!/bin/sh\nprintf '3.15.2 cpython arm64\\n'\n");
    let mismatch = invoke(&["toolbox", "status"]);
    assert_eq!(mismatch["library"]["runtimes"][0]["available"], true);
    assert_eq!(mismatch["library"]["runtimes"][0]["compatible"], false);
    assert_eq!(mismatch["library"]["runtimes"][0]["ready_offline"], false);
    fixture.write("python3", "#!/bin/sh\nprintf 'not a version\\n'\n");
    let invalid = invoke(&["toolbox", "status"]);
    assert_eq!(invalid["library"]["runtimes"][0]["available"], false);
    assert_eq!(invalid["library"]["runtimes"][0]["ready_offline"], false);
}
