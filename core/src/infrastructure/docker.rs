use super::{archive, checkpoint, stage, Job, Service};
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

pub(super) struct Docker {
    executable: PathBuf,
    endpoint: String,
    platform: String,
}

fn executable() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("DEVLINE_DOCKER") {
        candidates.push(PathBuf::from(path));
    } else {
        if let Some(home) = env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
        {
            candidates.extend([
                home.join(".docker/bin/docker"),
                home.join(".orbstack/bin/docker"),
            ]);
        }
        candidates.extend(
            env::split_paths(&env::var_os("PATH").unwrap_or_default())
                .filter(|path| path.is_absolute())
                .map(|path| path.join("docker")),
        );
        candidates.extend(
            [
                "/opt/homebrew/bin/docker",
                "/usr/local/bin/docker",
                "/Applications/Docker.app/Contents/Resources/bin/docker",
                "/Applications/OrbStack.app/Contents/MacOS/xbin/docker",
            ]
            .into_iter()
            .map(PathBuf::from),
        );
    }
    candidates
        .into_iter()
        .filter(|path| path.is_absolute())
        .find_map(|path| {
            let resolved = path.canonicalize().ok()?;
            let metadata = fs::metadata(&resolved).ok()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 == 0 {
                    return None;
                }
            }
            metadata.is_file().then_some(path)
        })
        .context("Docker CLI is unavailable. Install Docker with Compose support.")
}

fn command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    for key in [
        "DOCKER_HOST",
        "DOCKER_CONTEXT",
        "DOCKER_TLS_VERIFY",
        "DOCKER_CERT_PATH",
        "DOCKER_API_VERSION",
        "DOCKER_DEFAULT_PLATFORM",
    ] {
        command.env_remove(key);
    }
    for (key, _) in env::vars_os().filter(|(key, _)| {
        let key = key.to_string_lossy();
        key.starts_with("COMPOSE_") || key.starts_with("DEVLINE_")
    }) {
        command.env_remove(key);
    }
    command.stdin(Stdio::null());
    command
}

fn local_endpoint(value: &str) -> Result<String> {
    let socket = value
        .strip_prefix("unix://")
        .context("Devline infrastructure uses a local Unix Docker socket")?;
    let socket = Path::new(socket);
    if !socket.is_absolute() {
        bail!("Docker socket must be an absolute local path");
    }
    let socket = socket
        .canonicalize()
        .context("The local Docker socket is unavailable")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if !fs::metadata(&socket)?.file_type().is_socket() {
            bail!("Docker endpoint is not a Unix socket");
        }
    }
    Ok(format!(
        "unix://{}",
        socket.to_str().context("Docker socket path is not UTF-8")?
    ))
}

fn capture(command: &mut Command, timeout: Duration) -> Result<(ExitStatus, String, String)> {
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(command, 0);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().context("Docker output unavailable")?;
    let stderr = child
        .stderr
        .take()
        .context("Docker diagnostics unavailable")?;
    fn read(input: impl Read) -> io::Result<String> {
        const LIMIT: u64 = 16 * 1024 * 1024;
        let mut bytes = Vec::new();
        input.take(LIMIT + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > LIMIT {
            return Err(io::Error::other("Docker output exceeded its limit"));
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    let stdout = std::thread::spawn(move || read(stdout));
    let stderr = std::thread::spawn(move || read(stderr));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            other => {
                crate::tasks::stop_group(&mut child);
                break match other {
                    Err(error) => Err(error.into()),
                    _ => Err(anyhow::anyhow!("Docker command timed out")),
                };
            }
        }
    };
    let out = stdout
        .join()
        .map_err(|_| anyhow::anyhow!("Docker output interrupted"))??;
    let err = stderr
        .join()
        .map_err(|_| anyhow::anyhow!("Docker diagnostics interrupted"))??;
    Ok((status?, out, err))
}

fn diagnostic(output: &str) -> String {
    let lines: Vec<_> = output.lines().rev().take(8).collect();
    lines
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .take(2048)
        .collect()
}

impl Docker {
    pub(super) fn discover() -> Result<Self> {
        let executable = executable()?;
        let selected = env::var("DEVLINE_DOCKER_SOCKET")
            .ok()
            .map(|socket| format!("unix://{socket}"));
        let endpoint = if let Some(endpoint) = selected {
            endpoint
        } else if let Some(host) = env::var("DOCKER_HOST")
            .ok()
            .filter(|_| env::var_os("DOCKER_CONTEXT").is_none())
        {
            host
        } else {
            let mut context = command(&executable);
            context.args([
                "context",
                "inspect",
                "--format",
                "{{json .Endpoints.docker.Host}}",
            ]);
            if let Some(name) = env::var_os("DOCKER_CONTEXT") {
                context.arg("--").arg(name);
            }
            let (status, stdout, stderr) = capture(&mut context, Duration::from_secs(5))?;
            if !status.success() {
                bail!(
                    "Could not inspect the local Docker context. {}",
                    diagnostic(&stderr)
                );
            }
            serde_json::from_str::<String>(stdout.trim())?
        };
        let endpoint = local_endpoint(&endpoint)?;
        let mut docker = Self {
            executable,
            endpoint,
            platform: String::new(),
        };
        let info = docker.read(
            &[
                "info",
                "--format",
                "{{json .OSType}} {{json .Architecture}}",
            ],
            Duration::from_secs(6),
        )?;
        let mut decoder = serde_json::Deserializer::from_str(&info).into_iter::<String>();
        let os = decoder
            .next()
            .context("Docker did not report its operating system")??;
        let arch = decoder
            .next()
            .context("Docker did not report its architecture")??;
        let arch = match arch.as_str() {
            "aarch64" | "arm64" => "arm64",
            "x86_64" | "amd64" => "amd64",
            other => other,
        };
        docker.platform = format!("{os}/{arch}");
        Ok(docker)
    }

    pub(super) fn platform(&self) -> &str {
        &self.platform
    }

    pub(super) fn require_platform(&self, platform: &str) -> Result<()> {
        if self.platform != platform {
            bail!(
                "Image targets {platform}. Local Docker runs {}.",
                self.platform
            );
        }
        Ok(())
    }

    fn command(&self) -> Command {
        let mut command = command(&self.executable);
        command.args(["--host", &self.endpoint]);
        command
    }

    pub(super) fn read(&self, args: &[&str], timeout: Duration) -> Result<String> {
        let (status, output, errors) = capture(self.command().args(args), timeout)?;
        if !status.success() {
            bail!("Docker command failed. {}", diagnostic(&errors));
        }
        Ok(output)
    }

    pub(super) fn run(&self, args: &[&str], label: &str, job: &mut Option<&mut Job>) -> Result<()> {
        let mut command = self.command();
        command.args(args);
        if let Some(job) = job.as_deref_mut() {
            return job.command(&mut command, label);
        }
        let (status, _, errors) = capture(&mut command, Duration::from_secs(30 * 60))?;
        if !status.success() {
            bail!("{label} failed. {}", diagnostic(&errors));
        }
        Ok(())
    }

    fn image(&self, name: &str) -> Result<Option<Value>> {
        let (status, output, errors) = capture(
            self.command()
                .args(["image", "inspect", "--format", "{{json .}}", name]),
            Duration::from_secs(6),
        )?;
        if !status.success() {
            if errors.to_ascii_lowercase().contains("no such image")
                || errors.to_ascii_lowercase().contains("no such object")
            {
                return Ok(None);
            }
            bail!("Could not inspect local image. {}", diagnostic(&errors));
        }
        Ok(Some(serde_json::from_str(output.trim())?))
    }

    fn matches(image: &Value, service: &Service) -> bool {
        let (os, arch) = service.image.platform.split_once('/').unwrap_or_default();
        image["Id"] == service.image.config_digest
            && image["Os"] == os
            && image["Architecture"] == arch
    }

    pub(super) fn image_matches(&self, name: &str, service: &Service) -> Result<bool> {
        Ok(self
            .image(name)?
            .is_some_and(|image| Self::matches(&image, service)))
    }

    pub(super) fn alias_loaded(&self, service: &Service) -> Result<bool> {
        match self.image(&service.alias())? {
            Some(image) if Self::matches(&image, service) => Ok(true),
            Some(_) => bail!("Devline image alias already refers to a different image"),
            None => Ok(false),
        }
    }

    pub(super) fn tag(&self, service: &Service, job: &mut Option<&mut Job>) -> Result<()> {
        if !self.alias_loaded(service)? {
            self.run(
                &[
                    "image",
                    "tag",
                    &service.image.config_digest,
                    &service.alias(),
                ],
                "Registering local image alias",
                job,
            )?;
        }
        if !self.alias_loaded(service)? {
            bail!("Docker did not register the verified image alias");
        }
        Ok(())
    }

    pub(super) fn images(&self) -> Result<BTreeMap<String, String>> {
        let output = self.read(
            &["image", "ls", "--no-trunc", "--format", "{{json .}}"],
            Duration::from_secs(8),
        )?;
        let mut rows = BTreeMap::new();
        for line in output.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line)?;
            if let (Some(repo), Some(tag), Some(id)) = (
                value["Repository"].as_str(),
                value["Tag"].as_str(),
                value["ID"].as_str(),
            ) {
                if repo.starts_with("devline.local/") {
                    rows.insert(format!("{repo}:{tag}"), id.to_owned());
                }
            }
        }
        Ok(rows)
    }

    pub(super) fn resource_labels(&self, kind: &str, name: &str) -> Result<Option<Value>> {
        let format = if kind == "container" {
            "{{json .Config.Labels}}"
        } else {
            "{{json .Labels}}"
        };
        let (status, output, errors) = capture(
            self.command()
                .args([kind, "inspect", "--format", format, name]),
            Duration::from_secs(6),
        )?;
        if status.success() {
            return Ok(Some(serde_json::from_str(output.trim())?));
        }
        let message = errors.to_ascii_lowercase();
        if message.contains("no such") || message.contains("not found") {
            return Ok(None);
        }
        bail!("Could not inspect Docker {kind}. {}", diagnostic(&errors))
    }

    pub(super) fn save(
        &self,
        service: &Service,
        destination: &Path,
        job: &mut Option<&mut Job>,
    ) -> Result<(String, u64)> {
        stage(job, "Saving and compressing the image")?;
        let mut command = self.command();
        command
            .args(["image", "save", &service.image.config_digest])
            .stdout(Stdio::piped());
        if let Some(job) = job.as_deref() {
            command.stderr(Stdio::from(job.log_writer()?));
        } else {
            command.stderr(Stdio::null());
        }
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command.spawn()?;
        let stdout = child
            .stdout
            .take()
            .context("Docker archive stream unavailable")?;
        let path = destination.to_path_buf();
        let writer = std::thread::spawn(move || archive::save_tar(stdout, &path));
        let deadline = Instant::now() + Duration::from_secs(30 * 60);
        let result = loop {
            let pending = checkpoint(job).and_then(|()| {
                if Instant::now() >= deadline {
                    bail!("Docker image save timed out");
                }
                if let Some(job) = job.as_deref_mut() {
                    job.progress(
                        "Saving and compressing the image",
                        fs::metadata(destination).map_or(0, |meta| meta.len()),
                        0,
                    )?;
                }
                Ok(())
            });
            if let Err(error) = pending {
                crate::tasks::stop_group(&mut child);
                break Err(error);
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break Ok(()),
                Ok(Some(status)) => {
                    break Err(anyhow::anyhow!("Docker image save failed ({status})"))
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(error) => {
                    crate::tasks::stop_group(&mut child);
                    break Err(error.into());
                }
            }
        };
        let archived = writer
            .join()
            .map_err(|_| anyhow::anyhow!("Docker archive stream interrupted"))?;
        result?;
        archived
    }
}

pub(super) fn report(docker: &Result<Docker>) -> Value {
    match docker {
        Ok(docker) => {
            json!({"available":true,"endpoint":docker.endpoint,"platform":docker.platform})
        }
        Err(error) => json!({"available":false,"detail":format!("{error:#}")}),
    }
}
