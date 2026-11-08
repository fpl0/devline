//! The app and CLI share one offline launch path and one owned process group.
use crate::{infrastructure, project, project_layout, project_prepare, text_arg, Context};
use anyhow::{bail, Context as _, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::{IpAddr, TcpListener, TcpStream},
    os::unix::{
        fs::{OpenOptionsExt, PermissionsExt},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

const POLICY: &str = "(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip \"localhost:*\"))";

#[derive(Default, Deserialize)]
struct Action {
    command: Vec<String>,
    url: Option<String>,
    health_path: Option<String>,
}
#[derive(Deserialize)]
struct Configuration {
    schema_version: u32,
    #[serde(default)]
    run: Action,
    #[serde(default)]
    test: Action,
}
#[derive(Deserialize, Serialize)]
struct ProcessRecord {
    pid: u32,
    identity: String,
    token: String,
    url: Option<String>,
    #[serde(default)]
    health_path: Option<String>,
}
struct Paths {
    project: PathBuf,
    directory: PathBuf,
}

impl Paths {
    fn new(project: PathBuf, runs: &Path) -> Self {
        let key = format!(
            "{:x}",
            Sha256::digest(project.as_os_str().as_encoded_bytes())
        );
        Self {
            project,
            directory: runs.join(key),
        }
    }
}

fn paths(ctx: &Context, request: &Value) -> Result<Paths> {
    let input = Path::new(text_arg(request, "path")?);
    let project = project::root(ctx, input)?;
    if !project.is_dir() {
        bail!("Choose a project folder");
    }
    let runs = crate::templates::library(ctx, Path::new("project-runs"), true)?;
    let input = input.canonicalize()?;
    let exact = if input.is_file() {
        input
            .parent()
            .context("Project file has no parent folder")?
    } else {
        &input
    };
    let mut path = Paths::new(project, &runs);
    if path.project != exact {
        // An earlier launch may be keyed to this exact directory, even if a
        // manifest edit now makes discovery select another project root.
        let previous = Paths::new(exact.to_owned(), &runs);
        if record(&previous).is_ok_and(|record| record.as_ref().is_some_and(owns)) {
            path = previous;
        }
    }
    fs::create_dir_all(&path.directory)?;
    fs::set_permissions(&path.directory, fs::Permissions::from_mode(0o700))?;
    Ok(path)
}

fn lock(path: &Paths) -> Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path.directory.join("lock"))?;
    file.try_lock_exclusive()
        .context("A project launch or stop is already in progress")?;
    Ok(file)
}

fn configuration_file(path: &Path) -> &'static str {
    if project_layout::is_organized(path) {
        project_layout::CONFIG
    } else {
        "devline.json"
    }
}

fn configuration(path: &Path) -> Result<Configuration> {
    let config_path = configuration_file(path);
    let file = path.join(config_path);
    let config = match fs::metadata(&file) {
        Ok(meta) => {
            if !meta.is_file() || meta.len() > 64 * 1024 {
                bail!("Invalid {config_path} launch configuration");
            }
            let value: Configuration = serde_json::from_slice(&fs::read(file)?)
                .with_context(|| format!("Read {config_path} launch configuration"))?;
            if value.schema_version != 1 {
                bail!("Unsupported {config_path} schema_version");
            }
            value
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let run = if path.join("run.sh").is_file() {
                vec!["/bin/sh".into(), "./run.sh".into()]
            } else if fs::read_dir(path)?.filter_map(Result::ok).any(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "csproj" || ext == "fsproj")
            }) {
                vec!["dotnet".into(), "run".into(), "--no-launch-profile".into()]
            } else {
                vec![]
            };
            let test = if path.join("test-offline.sh").is_file() {
                vec!["/bin/sh".into(), "./test-offline.sh".into()]
            } else {
                vec![]
            };
            Configuration {
                schema_version: 1,
                run: Action {
                    command: run,
                    url: None,
                    health_path: None,
                },
                test: Action {
                    command: test,
                    url: None,
                    health_path: None,
                },
            }
        }
        Err(error) => return Err(error.into()),
    };
    for action in [&config.run, &config.test] {
        if action.command.len() > 128
            || action
                .command
                .iter()
                .any(|s| s.is_empty() || s.len() > 8192 || s.contains('\0'))
        {
            bail!("Invalid command arguments in {config_path}");
        }
    }
    if let Some(url) = &config.run.url {
        loopback_address(url)?;
    }
    if let Some(health) = &config.run.health_path {
        if config.run.url.is_none()
            || !health.starts_with('/')
            || health.starts_with("//")
            || health.bytes().any(|b| b <= 32 || b == 127)
        {
            bail!("run.health_path needs a local app URL and an absolute path without whitespace");
        }
    }
    Ok(config)
}

fn loopback_address(url: &str) -> Result<std::net::SocketAddr> {
    let rest = url
        .strip_prefix("http://")
        .context("The project app URL must use http:// on a loopback address")?;
    let authority = rest.split('/').next().unwrap_or_default();
    let (host, port) = authority
        .rsplit_once(':')
        .context("Include a local port in the project app URL")?;
    let ip: IpAddr = if host == "localhost" {
        "127.0.0.1".parse()?
    } else {
        host.trim_matches(['[', ']'])
            .parse()
            .context("Use localhost or a loopback IP for the app URL")?
    };
    let port: u16 = port.parse()?;
    if !ip.is_loopback() || port == 0 {
        bail!("The project app URL must use a loopback address and valid port");
    }
    Ok(std::net::SocketAddr::new(ip, port))
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn commands(ctx: &Context, request: &Value) -> Result<Value> {
    let path = paths(ctx, request)?;
    let config = configuration(&path.project)?;
    let runner = project_prepare::runner(ctx, &json!({"path":path.project}));
    let launchable = !config.run.command.is_empty() && runner.is_ok();
    let generated = project_layout::is_organized(&path.project);
    let launcher = quote(&path.project.join("devline.sh").to_string_lossy());
    let prefix = format!(
        "devline --offline project --path {}",
        quote(&path.project.to_string_lossy())
    );
    let test = (!config.test.command.is_empty()).then(|| {
        if generated && config.test.command == ["./devline.sh", "test"] {
            return format!("{launcher} test");
        }
        format!(
            "{prefix} run -- {}",
            config
                .test
                .command
                .iter()
                .map(|s| quote(s))
                .collect::<Vec<_>>()
                .join(" ")
        )
    });
    let run = launchable.then(|| {
        if generated {
            format!("{launcher} run")
        } else {
            format!("{prefix} run")
        }
    });
    let config_path = configuration_file(&path.project);
    let guide = path.project.join("README.md");
    Ok(
        json!({"path":path.project,"launchable":launchable,"run_command":run,"test_command":test,"guide_path":guide.is_file().then_some(guide),"detail":match runner {Err(error)=>format!("{error:#}"),_ if config.run.command.is_empty()=>format!("Add run.command to {config_path} to run this project from Devline."),_=>"Run with prepared dependencies and local service connections. Internet access is blocked; local services remain reachable.".into()}}),
    )
}

fn environment(ctx: &Context, project: &Path) -> Result<BTreeMap<String, String>> {
    let mut request = json!({"command":"infra.stack.config","path":project});
    let config = infrastructure::execute(ctx, &request, None)?;
    if config["configured"] != true {
        return Ok(BTreeMap::new());
    }
    request["command"] = json!("infra.stack.credentials");
    let credentials = infrastructure::execute(ctx, &request, None)?;
    serde_json::from_value(credentials["environment"].clone())
        .context("Read local service connection settings")
}

fn command(ctx: &Context, project: &Path, arguments: &[String]) -> Result<Command> {
    let runner = project_prepare::runner(ctx, &json!({"path":project}))?;
    if Path::new(text_arg(&runner, "path")?) != project {
        bail!("The project root changed. Select the project again.");
    }
    let mut command = Command::new("/usr/bin/sandbox-exec");
    crate::tasks::normalize_child_signals(&mut command);
    command
        .args(["-p", POLICY, "/bin/sh"])
        .arg(text_arg(&runner, "adapter")?)
        .arg("run")
        .arg(project)
        .arg(text_arg(&runner, "cache")?)
        .arg(text_arg(&runner, "toolchain_path")?)
        .args(arguments)
        .current_dir(project)
        .envs(environment(ctx, project)?);
    Ok(command)
}

fn set_launch_url(command: &mut Command, address: std::net::SocketAddr) {
    let origin = format!("http://{address}");
    command
        .env("DEVLINE_APP_URL", &origin)
        .env("ASPNETCORE_URLS", &origin);
}

fn run_arguments<'a>(action: &'a Action, project: &Path) -> Result<&'a [String]> {
    if action.command.is_empty() {
        bail!(
            "Add run.command to {}, or specify a command after --",
            configuration_file(project)
        );
    }
    Ok(&action.command)
}

pub fn foreground(request: &Value, supplied: &[String]) -> Result<i32> {
    let ctx = Context::new(request)?;
    let path = project::root(&ctx, Path::new(text_arg(request, "path")?))?;
    let config = supplied
        .is_empty()
        .then(|| configuration(&path))
        .transpose()?;
    let arguments = match &config {
        Some(config) => run_arguments(&config.run, &path)?,
        None => supplied,
    };
    let mut app = command(&ctx, &path, arguments)?;
    if let Some(config) = config {
        if let Some(url) = config.run.url {
            set_launch_url(&mut app, loopback_address(&url)?);
        }
        start_services(&ctx, &path)?;
    }
    Ok(app
        .status()
        .context("Run the offline project")?
        .code()
        .unwrap_or(1))
}

fn identity(pid: u32) -> Option<String> {
    let output = Command::new("/bin/ps")
        .args([
            "-ww",
            "-p",
            &pid.to_string(),
            "-o",
            "stat=",
            "-o",
            "lstart=",
            "-o",
            "command=",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    let mut fields = text.trim().splitn(2, char::is_whitespace);
    if fields.next()?.contains('Z') {
        return None;
    }
    fields
        .next()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}
fn owns(record: &ProcessRecord) -> bool {
    identity(record.pid)
        .is_some_and(|value| value == record.identity && value.contains(&record.token))
        && unsafe { libc::getpgid(record.pid as i32) } == record.pid as i32
}
fn record(path: &Paths) -> Result<Option<ProcessRecord>> {
    match fs::read(path.directory.join("process.json")) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}
fn status_at(path: &Paths) -> Result<Value> {
    let record = record(path)?;
    let running = record.as_ref().is_some_and(owns);
    let ready = running && record.as_ref().is_some_and(app_ready);
    let exit = fs::read_to_string(path.directory.join("exit"))
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok());
    let failed = !running && exit.is_some_and(|code| code != 0);
    Ok(
        json!({"path":path.project,"running":running,"state":if ready {"running"} else if running {"starting"} else if failed {"failed"} else {"stopped"},"pid":record.as_ref().filter(|_|running).map(|r|r.pid),"url":record.and_then(|r|r.url),"exit_code":exit,"detail":if ready {"App is running with internet access blocked.".into()} else if running {"Waiting for the app to build and become ready. Open App Logs for progress.".into()} else if failed {format!("App exited with code {}. Open App Logs for details.",exit.unwrap_or(1))} else {"App is stopped. Local services and their data are kept.".into()}}),
    )
}

fn app_ready(record: &ProcessRecord) -> bool {
    let Some(url) = &record.url else {
        return true;
    };
    let Ok(address) = loopback_address(url) else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(150)) else {
        return false;
    };
    let Some(path) = &record.health_path else {
        return true;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(150)));
    if write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .is_err()
    {
        return false;
    }
    let mut line = String::new();
    BufReader::new(stream.take(1024))
        .read_line(&mut line)
        .is_ok()
        && line.split_whitespace().nth(1) == Some("200")
}

pub(crate) fn status(ctx: &Context, request: &Value) -> Result<Value> {
    status_at(&paths(ctx, request)?)
}

fn start_services(ctx: &Context, project: &Path) -> Result<()> {
    let mut stack = json!({"command":"infra.stack.config","path":project});
    if infrastructure::execute(ctx, &stack, None)?["configured"] == true {
        stack["command"] = json!("infra.stack.up");
        infrastructure::execute(ctx, &stack, None)?;
    }
    Ok(())
}

pub(crate) fn start(ctx: &Context, request: &Value) -> Result<Value> {
    let path = paths(ctx, request)?;
    let _lock = lock(&path)?;
    if record(&path)?.as_ref().is_some_and(owns) {
        return status_at(&path);
    }
    let config = configuration(&path.project)?;
    let args = run_arguments(&config.run, &path.project)?;
    let mut app = command(ctx, &path.project, args)?;
    if let Some(url) = &config.run.url {
        let address = loopback_address(url)?;
        TcpListener::bind(address).with_context(||format!("The app port in {url} is already in use. Stop that app or change run.url and its matching launch setting in {}.", configuration_file(&path.project)))?;
        set_launch_url(&mut app, address);
    }
    start_services(ctx, &path.project)?;
    let mut random = [0u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut random)?;
    let token = format!(
        "devline-run-{}",
        random
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let exit_file = path.directory.join("exit");
    if exit_file.exists() {
        fs::remove_file(&exit_file)?;
    }
    let log = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path.directory.join("app.log"))?;
    // The supervisor keeps a unique argv identity while waiting for the whole
    // launch. Stop validates this identity before signalling its process group.
    // After the immediate child exits, preserve its result and clean up remaining
    // descendants before relinquishing ownership. Killing our own still-live
    // group avoids both orphan listeners and signalling a reused process ID.
    let mut supervisor = Command::new("/bin/sh");
    crate::tasks::normalize_child_signals(&mut supervisor);
    supervisor.args(["-c", "exit_file=$1; shift; trap 'while :; do /bin/sleep 1; done' TERM INT; \"$@\" & child=$!; wait \"$child\"; result=$?; printf '%s' \"$result\" > \"$exit_file\"; trap '' TERM INT; kill -TERM -- -$$; /bin/sleep 0.25; kill -KILL -- -$$", &token])
        .arg(&exit_file).arg(app.get_program()).args(app.get_args()).envs(app.get_envs().filter_map(|(k,v)|v.map(|v|(k,v))))
        .current_dir(&path.project).stdin(Stdio::null()).stdout(log.try_clone()?).stderr(log);
    unsafe {
        supervisor.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = supervisor.spawn().context("Start the offline project")?;
    let pid = child.id();
    let id = (0..20).find_map(|_| {
        let value = identity(pid);
        if value.as_ref().is_some_and(|s| s.contains(&token)) {
            value
        } else {
            thread::sleep(Duration::from_millis(10));
            None
        }
    });
    let Some(identity) = id else {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        let _ = child.kill();
        let _ = child.wait();
        bail!("App exited during launch. Open App Logs for details.");
    };
    let record = ProcessRecord {
        pid,
        identity,
        token,
        url: config.run.url,
        health_path: config.run.health_path,
    };
    let publish = (|| -> Result<()> {
        let temporary = path.directory.join("process.pending");
        fs::write(&temporary, serde_json::to_vec(&record)?)?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))?;
        fs::rename(temporary, path.directory.join("process.json"))?;
        Ok(())
    })();
    if let Err(error) = publish {
        if owns(&record) {
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        return Err(error).context("Could not record the launched app; the app has been stopped");
    }
    thread::spawn(move || {
        let _ = child.wait();
    });
    status_at(&path)
}

pub(crate) fn stop(ctx: &Context, request: &Value) -> Result<Value> {
    let path = paths(ctx, request)?;
    let _lock = lock(&path)?;
    if let Some(record) = record(&path)? {
        if owns(&record) {
            unsafe {
                libc::kill(-(record.pid as i32), libc::SIGTERM);
            }
            for _ in 0..40 {
                if !owns(&record) {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            if owns(&record) {
                unsafe {
                    libc::kill(-(record.pid as i32), libc::SIGKILL);
                }
                for _ in 0..40 {
                    if !owns(&record) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
    // An intentional stop is not presented as an app crash.
    fs::write(path.directory.join("exit"), "0")?;
    status_at(&path)
}

pub(crate) fn logs(ctx: &Context, request: &Value) -> Result<Value> {
    let path = paths(ctx, request)?;
    let limit = request["tail"].as_u64().unwrap_or(200).clamp(1, 2000) as usize;
    let file = path.directory.join("app.log");
    let mut text = String::new();
    if file.exists() {
        let mut file = File::open(file)?;
        let size = file.metadata()?.len();
        file.seek(SeekFrom::Start(size.saturating_sub(256 * 1024)))?;
        let mut bytes = Vec::new();
        file.take(256 * 1024).read_to_end(&mut bytes)?;
        text = String::from_utf8_lossy(&bytes).into_owned();
    }
    let mut lines = text.lines().rev().take(limit).collect::<Vec<_>>();
    lines.reverse();
    Ok(json!({"path":path.project,"log":lines.join("\n")}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_starters_declare_valid_launches() -> Result<()> {
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data");
        let catalog: crate::Catalog =
            serde_json::from_slice(&fs::read(data.join("catalog.json"))?)?;
        for starter in catalog.templates {
            let label = format!("{}@{}", starter.id, starter.version);
            let project = data.join(starter.path).join("project");
            let config =
                configuration(&project).with_context(|| format!("Read launch for {label}"))?;
            assert!(
                !config.run.command.is_empty(),
                "{label} needs a run command"
            );
            for argument in &config.run.command {
                if argument.starts_with("./") {
                    assert!(
                        project.join(argument).is_file(),
                        "{label} is missing {argument}"
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn run_urls_are_loopback_only() {
        for url in [
            "http://localhost:5080",
            "http://127.0.0.1:5080/",
            "http://[::1]:5080",
        ] {
            assert!(loopback_address(url).is_ok(), "{url}");
        }
        for url in [
            "https://example.com",
            "http://0.0.0.0:5080",
            "http://example.com:80",
            "file:///tmp/app",
            "http://localhost:0",
            "http://localhost@evil.test:80",
        ] {
            assert!(loopback_address(url).is_err(), "{url}");
        }
    }
    #[test]
    fn reused_or_unowned_process_is_never_signalled() {
        let record = ProcessRecord {
            pid: std::process::id(),
            identity: identity(std::process::id()).unwrap(),
            token: "not-a-devline-launch".into(),
            url: None,
            health_path: None,
        };
        assert!(!owns(&record));
    }
}
