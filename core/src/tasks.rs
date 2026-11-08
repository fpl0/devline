use crate::{offline, templates, text_arg, Context};
use anyhow::{bail, Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DEFAULT_TIMEOUT: u64 = 30 * 60;
const MAX_LOG_READ: u64 = 64 * 1024;
static SERIAL: AtomicU64 = AtomicU64::new(0);

/// A native host can ignore or block SIGCHLD. Preserve its signal policy in the
/// app, but give launched tools normal child-exit delivery before exec. .NET's
/// process watcher otherwise hangs waiting for compiler children to finish.
pub(crate) fn normalize_child_signals(command: &mut Command) {
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        command.pre_exec(|| {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = libc::SIG_DFL;
            if libc::sigemptyset(&mut action.sa_mask) == -1
                || libc::sigaction(libc::SIGCHLD, &action, std::ptr::null_mut()) == -1
            {
                return Err(std::io::Error::last_os_error());
            }
            let mut child_signal: libc::sigset_t = std::mem::zeroed();
            if libc::sigemptyset(&mut child_signal) == -1
                || libc::sigaddset(&mut child_signal, libc::SIGCHLD) == -1
                || libc::sigprocmask(libc::SIG_UNBLOCK, &child_signal, std::ptr::null_mut()) == -1
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum State {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl State {
    fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

#[derive(Deserialize, Serialize)]
struct Status {
    id: String,
    kind: String,
    #[serde(default)]
    target_path: Option<String>,
    #[serde(default)]
    target_reference: Option<String>,
    state: State,
    stage: String,
    created_at: u64,
    started_at: Option<u64>,
    updated_at: u64,
    result: Option<Value>,
    error: Option<String>,
    worker_pid: Option<u32>,
    completed_bytes: Option<u64>,
    total_bytes: Option<u64>,
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn jobs(ctx: &Context) -> Result<PathBuf> {
    let path = ctx.home.canonicalize()?.join("jobs");
    match fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    if !fs::symlink_metadata(&path)?.is_dir() {
        bail!("Invalid job storage");
    }
    Ok(path)
}

fn job_path(ctx: &Context, id: &str) -> Result<PathBuf> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        bail!("Invalid job ID");
    }
    let path = jobs(ctx)?.join(id);
    if !fs::symlink_metadata(&path)?.is_dir() {
        bail!("Invalid job directory");
    }
    Ok(path)
}

fn open_regular(path: &Path) -> Result<File> {
    if !fs::symlink_metadata(path)?.is_file() {
        bail!("Invalid job file");
    }
    Ok(File::open(path)?)
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_reader(open_regular(path)?)?)
}

fn save(path: &Path, status: &Status) -> Result<()> {
    let temporary = path.join("status.tmp");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer(&mut file, status)?;
    file.sync_data()?;
    fs::rename(temporary, path.join("status.json"))?;
    Ok(())
}

fn timeout(request: &Value) -> Result<Duration> {
    let seconds = request
        .get("timeout_seconds")
        .map(|value| {
            value
                .as_u64()
                .context("Job timeout must be a whole number of seconds")
        })
        .transpose()?
        .unwrap_or(DEFAULT_TIMEOUT);
    if !(1..=7200).contains(&seconds) {
        bail!("Job timeout must be between 1 and 7200 seconds");
    }
    Ok(Duration::from_secs(seconds))
}

pub fn start(ctx: &Context, request: &Value) -> Result<Value> {
    let kind = text_arg(request, "kind")?;
    match kind {
        "new_project" => {
            text_arg(request, "id")?;
            text_arg(request, "destination")?;
        }
        "build" | "verify" | "prepare" | "docs_install" | "docs_prepare" | "infra_prepare"
        | "infra_restore" | "infra_verify" | "infra_update" => {
            text_arg(request, "id")?;
        }
        "bundle_export"
        | "bundle_import"
        | "infra_export"
        | "infra_import"
        | "infra_stack_up"
        | "infra_stack_down"
        | "infra_stack_add"
        | "service_clients_add"
        | "project_prepare"
        | "toolbox_prepare" => {
            text_arg(request, "path")?;
        }
        "content_check" | "content_update" => {}
        _ => bail!("Unknown job kind {kind}"),
    }
    timeout(request)?;
    let exe = std::env::current_exe()?;
    let worker = if exe.file_name().is_some_and(|name| name == "devline") {
        exe
    } else {
        exe.with_file_name("devline")
    };
    if !worker.is_file() {
        bail!("The Devline job worker is missing from the app bundle");
    }
    let id = format!(
        "{:x}-{:x}-{:x}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    );
    let path = jobs(ctx)?.join(&id);
    fs::create_dir(&path)?;
    let started = (|| -> Result<Value> {
        let mut request = request.clone();
        if kind == "new_project" {
            let destination = std::path::absolute(text_arg(&request, "destination")?)?;
            request["destination"] = json!(destination);
            request["path"] = json!(destination);
        }
        if let Some(input) = request.get_mut("path") {
            if let Some(value) = input.as_str() {
                let absolute =
                    std::fs::canonicalize(value).or_else(|_| std::path::absolute(value))?;
                *input = json!(absolute);
            }
        }
        fs::write(path.join("request.json"), serde_json::to_vec(&request)?)?;
        File::create(path.join("job.log"))?;
        let record = Status {
            id: id.clone(),
            kind: kind.to_owned(),
            target_path: request["path"].as_str().map(str::to_owned),
            target_reference: request["id"].as_str().map(str::to_owned),
            state: State::Queued,
            stage: "Queued".into(),
            created_at: now(),
            started_at: None,
            updated_at: now(),
            result: None,
            error: None,
            worker_pid: None,
            completed_bytes: None,
            total_bytes: None,
        };
        save(&path, &record)?;
        let mut command = Command::new(worker);
        command
            .arg("__job-worker")
            .arg(&id)
            .arg(ctx.home.canonicalize()?)
            .arg(ctx.catalog_dir.canonicalize()?)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        normalize_child_signals(&mut command);
        let mut child = command.spawn().context("Start the Devline job worker")?;
        // Reap in long-lived desktop processes. The detached worker survives CLI exit.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        read_status(&path, None)
    })();
    if started.is_err() {
        let _ = fs::remove_dir_all(&path);
    }
    started
}

fn read_status(path: &Path, offset: Option<u64>) -> Result<Value> {
    let mut record: Status = read(&path.join("status.json"))?;
    #[cfg(unix)]
    if record.state == State::Running
        && record.worker_pid.is_some_and(|pid| {
            let Ok(pid) = i32::try_from(pid) else {
                return true;
            };
            (unsafe { libc::kill(pid, 0) }) == -1
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        })
    {
        record.state = State::Failed;
        record.stage = "Interrupted".into();
        record.error = Some("The worker exited before completing the job.".into());
    }
    if record.state == State::Queued && now().saturating_sub(record.created_at) > 60 {
        record.state = State::Failed;
        record.stage = "Interrupted".into();
        record.error = Some("The worker did not start.".into());
    }
    let active = record.state.active();
    let mut result = serde_json::to_value(record)?;
    let log_path = path.join("job.log");
    result["log_path"] = json!(log_path);
    if let Some(offset) = offset {
        let mut file = open_regular(&log_path)?;
        let offset = offset.min(file.metadata()?.len());
        file.seek(SeekFrom::Start(offset))?;
        let mut bytes = Vec::new();
        file.take(MAX_LOG_READ).read_to_end(&mut bytes)?;
        let end = if active || bytes.len() as u64 == MAX_LOG_READ {
            complete_utf8_prefix(&bytes)
        } else {
            bytes.len()
        };
        result["log"] = json!(String::from_utf8_lossy(&bytes[..end]));
        result["offset"] = json!(offset);
        result["next_offset"] = json!(offset + end as u64);
    }
    Ok(result)
}

fn complete_utf8_prefix(bytes: &[u8]) -> usize {
    // A partial UTF-8 character spans at most three trailing bytes. Check
    // that suffix independently so earlier malformed output cannot hide it.
    let tail = bytes.len().saturating_sub(3);
    if let Some(index) = bytes[tail..].iter().rposition(|byte| byte & 0xc0 != 0x80) {
        let start = tail + index;
        if std::str::from_utf8(&bytes[start..]).is_err_and(|error| error.error_len().is_none()) {
            return start;
        }
    }
    bytes.len()
}

pub fn status(ctx: &Context, request: &Value) -> Result<Value> {
    let path = job_path(ctx, text_arg(request, "job_id")?)?;
    read_status(&path, Some(request["offset"].as_u64().unwrap_or(0)))
}

pub fn cancel(ctx: &Context, request: &Value) -> Result<Value> {
    let path = job_path(ctx, text_arg(request, "job_id")?)?;
    let record: Status = read(&path.join("status.json"))?;
    if record.state.active() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join("cancel"))
        {
            Ok(file) => file.sync_data()?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    let mut result = read_status(&path, None)?;
    result["cancel_requested"] = json!(path.join("cancel").exists());
    Ok(result)
}

pub fn list(ctx: &Context) -> Result<Value> {
    let mut records = Vec::new();
    for entry in fs::read_dir(jobs(ctx)?)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Ok(record) = read_status(&entry.path(), None) {
                records.push(record);
            }
        }
    }
    records.sort_by(|a, b| b["id"].as_str().cmp(&a["id"].as_str()));
    Ok(json!(records))
}

pub(crate) struct Job {
    path: PathBuf,
    record: Status,
    deadline: Instant,
    log: File,
    last_progress: Instant,
}

impl Job {
    pub(crate) fn log_writer(&self) -> Result<File> {
        Ok(self.log.try_clone()?)
    }
    pub(crate) fn checkpoint(&self) -> Result<()> {
        if self.path.join("cancel").exists() {
            bail!("Job cancelled");
        }
        if Instant::now() >= self.deadline {
            bail!("Job timed out");
        }
        Ok(())
    }

    pub(crate) fn stage(&mut self, label: &str) -> Result<()> {
        self.checkpoint()?;
        self.record.stage = label.to_owned();
        self.record.updated_at = now();
        self.record.completed_bytes = None;
        self.record.total_bytes = None;
        save(&self.path, &self.record)?;
        writeln!(self.log, "\n==> {label}")?;
        Ok(())
    }

    pub(crate) fn progress(&mut self, label: &str, completed: u64, total: u64) -> Result<()> {
        self.checkpoint()?;
        let changed = self.record.stage != label;
        if changed {
            writeln!(self.log, "\n==> {label}")?;
        }
        if changed
            || completed == total
            || self.last_progress.elapsed() >= Duration::from_millis(150)
        {
            self.record.stage = label.to_owned();
            self.record.completed_bytes = Some(completed);
            self.record.total_bytes = Some(total);
            self.record.updated_at = now();
            save(&self.path, &self.record)?;
            self.last_progress = Instant::now();
        }
        Ok(())
    }

    pub(crate) fn command(&mut self, command: &mut Command, label: &str) -> Result<()> {
        self.stage(label)?;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::from(self.log.try_clone()?))
            .stderr(Stdio::from(self.log.try_clone()?));
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(command, 0);
        normalize_child_signals(command);
        let mut child = command
            .spawn()
            .with_context(|| format!("Could not start {label}"))?;
        loop {
            if let Err(reason) = self.checkpoint() {
                stop_group(&mut child);
                return Err(reason);
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(status)) => bail!("{label} failed ({status}). See the full job log."),
                Ok(None) => std::thread::sleep(Duration::from_millis(60)),
                Err(error) => {
                    stop_group(&mut child);
                    return Err(error.into());
                }
            }
        }
    }
}

pub(crate) fn stop_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    if let Ok(pid) = i32::try_from(child.id()) {
        unsafe { libc::kill(-pid, libc::SIGTERM) };
        // The main shell may exit before its descendants. Escalate the whole group.
        std::thread::sleep(Duration::from_millis(250));
        unsafe { libc::kill(-pid, libc::SIGKILL) };
    }
    let _ = child.kill();
    let _ = child.wait();
}

pub fn run_worker(home: PathBuf, catalog: PathBuf, id: &str) -> Result<()> {
    let ctx = Context::new(&json!({"home":home,"catalog":catalog}))?;
    let path = job_path(&ctx, id)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.join("worker.lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock).context("Job already has a worker")?;
    let mut record: Status = read(&path.join("status.json"))?;
    if record.state != State::Queued || record.id != id {
        bail!("Job cannot be started");
    }
    let request: Value = read(&path.join("request.json"))?;
    let log = OpenOptions::new().append(true).open(path.join("job.log"))?;
    record.worker_pid = Some(std::process::id());
    record.state = State::Running;
    record.started_at = Some(now());
    let mut job = Job {
        path,
        record,
        deadline: Instant::now() + timeout(&request)?,
        log,
        last_progress: Instant::now(),
    };
    let kind = job.record.kind.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<Value> {
        job.stage("Preparing job")?;
        match kind.as_str() {
            "new_project" => {
                crate::project_prepare::create_with_job(&ctx, &request, Some(&mut job))
            }
            "build" => templates::build_latest_job(&ctx, text_arg(&request, "id")?, Some(&mut job)),
            "verify" => templates::verify_offline(&ctx, text_arg(&request, "id")?, Some(&mut job)),
            "prepare" => templates::prepare(&ctx, text_arg(&request, "id")?, Some(&mut job)),
            "bundle_export" => offline::export_job(&ctx, &request, Some(&mut job)),
            "bundle_import" => offline::import_job(&ctx, &request, Some(&mut job)),
            "docs_install" => {
                job.stage("Installing documentation")?;
                crate::doc_updates::install_with_progress(
                    &ctx,
                    text_arg(&request, "id")?,
                    request["version"].as_str(),
                    |label, completed, total| job.progress(label, completed, total),
                )
            }
            "docs_prepare" => {
                job.stage("Installing local documentation")?;
                crate::doc_library::install_docs_job(
                    &ctx,
                    text_arg(&request, "id")?,
                    request["version"].as_str(),
                    Some(&mut job),
                )
            }
            "project_prepare" => crate::project_prepare::prepare(&ctx, &request, Some(&mut job)),
            "toolbox_prepare" => crate::toolbox::prepare(&ctx, &request, Some(&mut job)),
            "service_clients_add" => crate::service_clients::add(&ctx, &request, Some(&mut job)),
            "content_check" => crate::content_updates::check(&ctx, &request, Some(&mut job)),
            "content_update" => crate::content_updates::apply(&ctx, &request, Some(&mut job)),
            kind if kind.starts_with("infra_") => {
                let command = match kind {
                    "infra_prepare" => "infra.prepare",
                    "infra_restore" => "infra.restore",
                    "infra_verify" => "infra.verify",
                    "infra_export" => "infra.export",
                    "infra_import" => "infra.import",
                    "infra_stack_up" => "infra.stack.up",
                    "infra_stack_down" => "infra.stack.down",
                    "infra_stack_add" => "infra.stack.add",
                    "infra_update" => "infra.update",
                    _ => bail!("Unknown infrastructure job"),
                };
                let mut request = request.clone();
                request["command"] = json!(command);
                crate::infrastructure::execute(&ctx, &request, Some(&mut job))
            }
            _ => bail!("Unknown job kind"),
        }
    }))
    .unwrap_or_else(|_| Err(anyhow::anyhow!("The worker could not finish the job.")));
    match result {
        Ok(result) => {
            let incomplete = (kind == "toolbox_prepare" && result["ready_offline"] == false)
                || (kind == "content_update" && result["complete"] == false)
                || (kind == "content_check" && result["all_checked"] == false);
            job.record.state = if incomplete {
                State::Failed
            } else {
                State::Succeeded
            };
            job.record.stage = if incomplete {
                "Needs attention"
            } else {
                "Complete"
            }
            .into();
            if incomplete {
                job.record.error = Some("Some resources still need attention. Review the result for completed steps and remaining blockers.".into());
            }
            job.record.result = Some(result);
        }
        Err(error) => {
            let cancelled = job.path.join("cancel").exists();
            job.record.state = if cancelled {
                State::Cancelled
            } else {
                State::Failed
            };
            job.record.stage = if cancelled { "Cancelled" } else { "Failed" }.into();
            job.record.error = Some(format!("{error:#}"));
            writeln!(job.log, "\n{}: {error:#}", job.record.stage)?;
        }
    }
    job.record.updated_at = now();
    save(&job.path, &job.record)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn log_pages_preserve_partial_utf8_after_malformed_bytes() {
        let text = b"start\xff\xf0\x9f\x91";
        assert_eq!(complete_utf8_prefix(text), b"start\xff".len());
        assert_eq!(
            complete_utf8_prefix(b"start\xff\xf0\x9f\x91\x8b"),
            text.len() + 1
        );
        assert_eq!(
            complete_utf8_prefix(b"start\xff\xc2?"),
            b"start\xff\xc2?".len()
        );
    }

    #[test]
    fn launched_tools_receive_child_exit_signals() -> Result<()> {
        use std::os::unix::process::CommandExt;
        let mut command = Command::new(std::env::current_exe()?);
        command
            .args([
                "--exact",
                "tasks::tests::child_exit_signal_probe",
                "--nocapture",
            ])
            .env("DEVLINE_CHILD_SIGNAL_PROBE", "1");
        // Model a native host's ignored disposition and a dispatch thread's
        // blocked signal without changing the test runner or its other threads.
        unsafe {
            command.pre_exec(|| {
                let mut action: libc::sigaction = std::mem::zeroed();
                action.sa_sigaction = libc::SIG_IGN;
                libc::sigemptyset(&mut action.sa_mask);
                if libc::sigaction(libc::SIGCHLD, &action, std::ptr::null_mut()) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                let mut mask: libc::sigset_t = std::mem::zeroed();
                libc::sigemptyset(&mut mask);
                libc::sigaddset(&mut mask, libc::SIGCHLD);
                if libc::sigprocmask(libc::SIG_BLOCK, &mask, std::ptr::null_mut()) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        normalize_child_signals(&mut command);
        let output = command.output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("CHILD_EXIT_SIGNALS_READY"));
        Ok(())
    }

    #[test]
    fn child_exit_signal_probe() {
        if std::env::var_os("DEVLINE_CHILD_SIGNAL_PROBE").is_none() {
            return;
        }
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            assert_eq!(
                libc::sigaction(libc::SIGCHLD, std::ptr::null(), &mut action),
                0
            );
            assert_eq!(action.sa_sigaction, libc::SIG_DFL);
            let mut mask: libc::sigset_t = std::mem::zeroed();
            assert_eq!(
                libc::sigprocmask(libc::SIG_BLOCK, std::ptr::null(), &mut mask),
                0
            );
            assert_eq!(libc::sigismember(&mask, libc::SIGCHLD), 0);
        }
        let mut child = Command::new("/bin/sh")
            .args(["-c", "exit 17"])
            .spawn()
            .unwrap();
        assert_eq!(child.wait().unwrap().code(), Some(17));
        println!("CHILD_EXIT_SIGNALS_READY");
    }

    #[test]
    fn cancellation_stops_descendants_and_keeps_streamed_output() -> Result<()> {
        let staging = crate::templates::Stage::new(&std::env::temp_dir())?;
        let path = staging.0.clone();
        let log = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(path.join("job.log"))?;
        let now = now();
        let mut job = Job {
            path: path.clone(),
            record: Status {
                id: "1".into(),
                kind: "build".into(),
                target_path: None,
                target_reference: None,
                state: State::Running,
                stage: String::new(),
                created_at: now,
                started_at: Some(now),
                updated_at: now,
                result: None,
                error: None,
                worker_pid: None,
                completed_bytes: None,
                total_bytes: None,
            },
            deadline: Instant::now() + Duration::from_secs(5),
            log,
            last_progress: Instant::now(),
        };
        let mut command = Command::new("/bin/sh");
        command.current_dir(&path).args(["-c",
            "printf 'starting refresh\\n'; (trap '' TERM; sleep 1; touch escaped) & touch started; wait",
        ]);
        std::thread::scope(|scope| -> Result<()> {
            scope.spawn(|| {
                let deadline = Instant::now() + Duration::from_secs(2);
                while !path.join("started").exists() && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(10));
                }
                fs::write(path.join("cancel"), "").unwrap();
            });
            let error = job.command(&mut command, "Refreshing").unwrap_err();
            assert!(error.to_string().contains("cancelled"));
            Ok(())
        })?;
        assert!(fs::read_to_string(path.join("job.log"))?.contains("starting refresh"));
        std::thread::sleep(Duration::from_secs(1));
        assert!(!path.join("escaped").exists());
        Ok(())
    }
}
