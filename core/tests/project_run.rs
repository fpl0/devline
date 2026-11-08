#![cfg(target_os = "macos")]
use serde_json::{json, Value};
use std::{
    fs,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    port: u16,
}
impl Fixture {
    fn new() -> Self {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let root = std::env::temp_dir()
            .join(format!(
                "devline-run-test-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ))
            .join("project");
        fs::create_dir_all(root.join(".devline-offline/wheelhouse")).unwrap();
        fs::write(root.join(".devline-offline/requirements.lock"), "").unwrap();
        fs::write(root.join("pyproject.toml"),"[project]\nname='offline-launch-proof'\nversion='0.0.0'\nrequires-python='>=3.9'\ndependencies=[]\n").unwrap();
        fs::write(root.join("devline.json"),json!({"schema_version":1,"run":{"command":["python3","server.py"],"url":format!("http://127.0.0.1:{port}"),"health_path":"/health"}}).to_string()).unwrap();
        fs::write(
            root.join("server.py"),
            r#"import errno, os, signal, socket
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlsplit
signal.signal(signal.SIGTERM, signal.SIG_IGN)
try:
    socket.create_connection(('1.1.1.1',443),timeout=1)
except OSError as error:
    assert error.errno in (errno.EPERM,errno.EACCES), error
    print('OUTBOUND_BLOCKED',flush=True)
else:
    raise RuntimeError('External network was allowed')
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200); self.end_headers(); self.wfile.write(b'offline')
origin = urlsplit(os.environ['DEVLINE_APP_URL'])
HTTPServer((origin.hostname, origin.port), Handler).serve_forever()
"#,
        )
        .unwrap();
        Self { root, port }
    }
    fn cli(&self, command: &str) -> Value {
        self.cli_at(&self.root, command)
    }
    fn cli_at(&self, path: &Path, command: &str) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_devline"))
            .args(["--json", "--home"])
            .arg(self.root.join("library"))
            .arg("--catalog")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../data"))
            .args(["project", "--path"])
            .arg(path)
            .arg(command)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{command} {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout).unwrap()["data"].clone()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_devline"))
            .arg("--home")
            .arg(self.root.join("library"))
            .arg("--catalog")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../data"))
            .args(["project", "--path"])
            .arg(&self.root)
            .arg("stop")
            .output();
        let _ = fs::remove_dir_all(self.root.parent().unwrap());
    }
}

#[test]
fn cli_launches_offline_and_stops_even_a_term_ignoring_listener() {
    let fixture = Fixture::new();
    let nested = fixture.root.join("src");
    fs::create_dir(&nested).unwrap();
    let manifest = fixture.root.join("pyproject.toml");
    assert_eq!(fixture.cli_at(&manifest, "commands")["launchable"], true);
    let first = fixture.cli_at(&nested, "start");
    assert_eq!(first["running"], true);
    assert_eq!(
        fixture.cli("start")["pid"],
        first["pid"],
        "Start must be idempotent"
    );
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        let status = fixture.cli("app-status");
        if status["state"] == "running" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "{status}; logs: {}",
            fixture.cli("logs")
        );
        thread::sleep(Duration::from_millis(150));
    }
    assert!(fixture.cli_at(&manifest, "logs")["log"]
        .as_str()
        .unwrap()
        .contains("OUTBOUND_BLOCKED"));
    assert!(TcpStream::connect(("127.0.0.1", fixture.port)).is_ok());
    fs::write(
        fixture.root.parent().unwrap().join("go.mod"),
        "module example.invalid/parent\n",
    )
    .unwrap();
    fs::remove_file(manifest).unwrap();
    assert_eq!(fixture.cli("app-status")["pid"], first["pid"]);
    assert_eq!(fixture.cli("start")["pid"], first["pid"]);
    assert!(fixture.cli("logs")["log"]
        .as_str()
        .unwrap()
        .contains("OUTBOUND_BLOCKED"));
    assert_eq!(fixture.cli("stop")["running"], false);
    let deadline = Instant::now() + Duration::from_secs(3);
    while TcpStream::connect(("127.0.0.1", fixture.port)).is_ok() {
        assert!(
            Instant::now() < deadline,
            "Stopped child still owns its port"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn normal_launcher_exit_cleans_up_its_term_ignoring_descendant() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root.join("launcher.py"),
        format!(
            r#"import socket, subprocess, sys, time
subprocess.Popen([sys.executable, 'server.py'])
for attempt in range(200):
    try:
        connection = socket.create_connection(('127.0.0.1', {port}), timeout=0.1)
        connection.close()
        print('DESCENDANT_LISTENING', flush=True)
        raise SystemExit(0)
    except OSError:
        time.sleep(0.025)
raise RuntimeError('Background listener did not start')
"#,
            port = fixture.port
        ),
    )
    .unwrap();
    fs::write(fixture.root.join("devline.json"), json!({"schema_version":1,"run":{"command":["python3","launcher.py"],"url":format!("http://127.0.0.1:{}",fixture.port)}}).to_string()).unwrap();
    assert_eq!(fixture.cli("start")["running"], true);
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        let status = fixture.cli("app-status");
        if status["running"] == false {
            assert_eq!(status["state"], "stopped");
            assert_eq!(status["exit_code"], 0);
            break;
        }
        assert!(
            Instant::now() < deadline,
            "{status}; {}",
            fixture.cli("logs")
        );
        thread::sleep(Duration::from_millis(50));
    }
    assert!(fixture.cli("logs")["log"]
        .as_str()
        .unwrap()
        .contains("DESCENDANT_LISTENING"));
    let deadline = Instant::now() + Duration::from_secs(3);
    while TcpStream::connect(("127.0.0.1", fixture.port)).is_ok() {
        assert!(
            Instant::now() < deadline,
            "Exited launcher left a background listener running"
        );
        thread::sleep(Duration::from_millis(25));
    }
    // Stop after natural exit remains harmless and idempotent.
    assert_eq!(fixture.cli("stop")["running"], false);
}
