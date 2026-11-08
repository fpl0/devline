use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

impl Fixture {
    fn new() -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "devline-infra-cli-{}-{suffix}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_devline"))
            .current_dir(&self.0)
            .arg("--home")
            .arg(self.0.join("library"))
            .arg("--catalog")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../data"))
            .arg("--json")
            .args(args)
            .output()
            .unwrap()
    }

    fn write_origin(metadata: &Path) {
        fs::create_dir(metadata).unwrap();
        fs::write(
            metadata.join("origin.json"),
            serde_json::json!({
                "template": "go/chi-api",
                "version": "1.2.0",
                "digest": "0".repeat(64),
            })
            .to_string(),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn offline_prepare_stops_before_accessing_the_library() {
    let fixture = Fixture::new();
    let output = fixture.run(&[
        "--offline",
        "infra",
        "prepare",
        "postgres@18.6",
        "--background",
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("infra restore postgres@18.6"));
    assert!(!fixture.0.join("library").exists());
}

#[test]
fn offline_configuration_and_project_summary_keep_credentials_private() {
    let fixture = Fixture::new();
    let project = fixture.0.join("project");
    fs::create_dir(&project).unwrap();
    let outside = fixture.0.join("other-project-metadata");
    Fixture::write_origin(&outside);
    std::os::unix::fs::symlink(&outside, project.join(".devline")).unwrap();
    let path = project.to_str().unwrap();
    let output = fixture.run(&[
        "--offline",
        "infra",
        "configure",
        path,
        "--service",
        "postgres@18.6",
        "--service",
        "redis@8.10.1",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["data"]["services"].as_array().unwrap().len(), 2);
    assert!(!outside.join("services").exists());
    let credentials = fs::read_to_string(project.join(".devline-infra/credentials.env")).unwrap();
    assert!(credentials.contains("DEVLINE_POSTGRES_PASSWORD="));
    let output = fixture.run(&["--offline", "project", "--path", path]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("PASSWORD"));
    let project: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(project["data"]["infrastructure"]["configured"], true);
}

#[test]
fn generated_projects_configure_services_under_devline() {
    let fixture = Fixture::new();
    let project = fixture.0.join("project");
    fs::create_dir(&project).unwrap();
    Fixture::write_origin(&project.join(".devline"));
    let path = project.to_str().unwrap();
    let output = fixture.run(&[
        "--offline",
        "infra",
        "configure",
        path,
        "--service",
        "postgres@18.6",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let service_path = project.join(".devline/services");
    assert!(service_path.join("compose.yaml").is_file());
    assert!(service_path.join("credentials.env").is_file());
    assert_eq!(
        fs::read_to_string(service_path.join(".gitignore")).unwrap(),
        "*\n"
    );
    assert!(!project.join(".devline-infra").exists());
    let output = fixture.run(&["--offline", "project", "--path", path]);
    assert!(output.status.success());
    let project: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(project["data"]["infrastructure"]["configured"], true);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("PASSWORD"));
}

#[test]
fn offline_service_additions_are_idempotent_and_export_client_connections_explicitly() {
    let fixture = Fixture::new();
    let project = fixture.0.join("project");
    fs::create_dir(&project).unwrap();
    let path = project.to_str().unwrap();
    let first = fixture.run(&[
        "--offline",
        "infra",
        "add",
        path,
        "--service",
        "postgres@18.6",
    ]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let original: Value = serde_json::from_slice(&first.stdout).unwrap();
    let before = fs::read_to_string(project.join(".devline-infra/credentials.env")).unwrap();
    for _ in 0..2 {
        let added = fixture.run(&[
            "--offline",
            "infra",
            "add",
            path,
            "--service",
            "rabbitmq@4.3.7",
        ]);
        assert!(
            added.status.success(),
            "{}",
            String::from_utf8_lossy(&added.stderr)
        );
        let added: Value = serde_json::from_slice(&added.stdout).unwrap();
        assert_eq!(
            added["data"]["project_name"],
            original["data"]["project_name"]
        );
        assert_eq!(added["data"]["services"].as_array().unwrap().len(), 2);
        assert!(!added.to_string().contains("PASSWORD"));
    }
    let after = fs::read_to_string(project.join(".devline-infra/credentials.env")).unwrap();
    for line in before.lines() {
        assert!(after.lines().any(|candidate| candidate == line));
    }
    let output = fixture.run(&["--offline", "infra", "env", path]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let environment: Value = serde_json::from_slice(&output.stdout).unwrap();
    let values = &environment["data"]["environment"];
    assert!(values["ConnectionStrings__Postgres"]
        .as_str()
        .unwrap()
        .starts_with("Host=127.0.0.1;"));
    assert_eq!(
        values["ConnectionStrings__RabbitMQ"],
        values["DEVLINE_RABBITMQ_URL"]
    );
    let output = Command::new(env!("CARGO_BIN_EXE_devline"))
        .arg("--home")
        .arg(fixture.0.join("library"))
        .arg("--catalog")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../data"))
        .args(["--offline", "infra", "env", path])
        .output()
        .unwrap();
    assert!(output.status.success());
    let shell = String::from_utf8(output.stdout).unwrap();
    assert!(shell.lines().all(|line| line.starts_with("export ")));
    assert!(shell.contains("export ConnectionStrings__Postgres='Host=127.0.0.1;"));
    assert!(!project
        .join(".devline-infra/pending-configuration.json")
        .exists());
}

#[test]
fn toolbox_reports_arbitrary_project_dependencies_unverified() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--offline", "toolbox", "check"]);
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["data"]["project_dependencies"]["state"],
        "unverified"
    );
    assert_eq!(report["data"]["ready_offline"], false);
}

#[test]
fn both_project_prepare_spellings_report_combined_readiness_and_exit_three() {
    let fixture = Fixture::new();
    let first = fixture.run(&["--offline", "toolbox", "prepare"]);
    let second = fixture.run(&["--offline", "project", "prepare"]);
    for output in [first, second] {
        assert_eq!(
            output.status.code(),
            Some(3),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["data"]["ready_offline"], false);
        assert!(report["data"]["library"].is_object());
        assert!(report["data"]["steps"].is_array());
        assert!(!report["data"]["blockers"].as_array().unwrap().is_empty());
    }
}

#[test]
fn incomplete_background_preparation_keeps_its_result_and_project_target() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--offline", "toolbox", "prepare", "--background"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let started: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(started["data"]["kind"], "toolbox_prepare");
    assert_eq!(
        started["data"]["target_path"],
        serde_json::json!(fixture.0.canonicalize().unwrap())
    );
    let id = started["data"]["id"].as_str().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let output = fixture.run(&["jobs", "status", id]);
        let state: Value = serde_json::from_slice(&output.stdout).unwrap();
        if !matches!(state["data"]["state"].as_str(), Some("queued" | "running")) {
            assert_eq!(state["data"]["state"], "failed");
            assert_eq!(state["data"]["stage"], "Needs attention");
            assert_eq!(state["data"]["result"]["ready_offline"], false);
            assert!(!state["data"]["result"]["blockers"]
                .as_array()
                .unwrap()
                .is_empty());
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "job did not complete: {state}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
