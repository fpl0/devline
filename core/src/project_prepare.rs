use crate::{project, safe_catalog_path, tasks::Job, templates, text_arg, Context};
use anyhow::{bail, Context as _, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

const MANAGED: &str = ".devline-offline";
const NETWORK_POLICY: &str =
    "(deny network-outbound)(allow network-outbound (remote ip \"localhost:*\"))";

#[derive(Deserialize, Serialize)]
struct Receipt {
    fingerprint: String,
    stack: String,
    toolchain: String,
    toolchain_path: PathBuf,
    verified_at: u64,
    #[serde(default)]
    generated: bool,
}

struct Project {
    path: PathBuf,
    stack: String,
    detected_stacks: Vec<String>,
}

impl Project {
    fn known(path: &Path, stack: &str) -> Self {
        Self {
            path: path.into(),
            stack: stack.into(),
            detected_stacks: vec![stack.into()],
        }
    }
    fn from_request(ctx: &Context, request: &Value) -> Result<Self> {
        let detected = project::context(ctx, Path::new(text_arg(request, "path")?))?;
        Self::from_profile(&detected)
    }

    fn from_profile(profile: &Value) -> Result<Self> {
        Ok(Self {
            path: PathBuf::from(text_arg(profile, "path")?),
            stack: profile["stack"].as_str().unwrap_or_default().to_owned(),
            detected_stacks: profile["stacks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
        })
    }

    fn unsupported_detail(&self) -> String {
        if self.detected_stacks.len() > 1 {
            format!("Multiple stacks detected ({}). Open each independently locked project directory to prepare it.",self.detected_stacks.join(", "))
        } else {
            "Open a project folder containing a supported runtime manifest.".into()
        }
    }

    fn adapter(&self, ctx: &Context) -> Result<PathBuf> {
        if self.stack.is_empty() {
            bail!("{}", self.unsupported_detail());
        }
        let path = safe_catalog_path(ctx, &format!("project-adapters/{}.sh", self.stack))
            .with_context(|| {
                format!("No offline project adapter is available for {}", self.stack)
            })?;
        if !fs::symlink_metadata(&path)?.is_file() {
            bail!("Invalid offline project adapter");
        }
        Ok(path)
    }

    fn toolchain(&self, ctx: &Context) -> Result<(String, PathBuf, String)> {
        if self.stack.is_empty() {
            bail!("{}", self.unsupported_detail());
        }
        let reports =
            templates::project_runtime_reports(ctx, &self.path, std::slice::from_ref(&self.stack))?;
        let report = reports
            .as_array()
            .and_then(|reports| reports.first())
            .context("No toolchain report is available")?;
        let detail = text_arg(report, "detail")?;
        let path = report["path"]
            .as_str()
            .filter(|_| report["available"] == true && report["compatible"] != false)
            .with_context(|| detail.to_owned())?;
        Ok((
            text_arg(report, "toolchain")?.to_owned(),
            PathBuf::from(path),
            detail.to_owned(),
        ))
    }

    fn key(&self) -> Result<String> {
        Ok(format!(
            "{:x}",
            Sha256::digest(
                self.path
                    .to_str()
                    .context("Project path is not UTF-8")?
                    .as_bytes()
            )
        ))
    }
}

fn records(ctx: &Context) -> Result<PathBuf> {
    templates::library(ctx, Path::new("project-readiness"), true)
}

fn receipt_path(ctx: &Context, project: &Project) -> Result<PathBuf> {
    Ok(
        templates::library(ctx, Path::new("project-readiness"), false)?
            .join(format!("{}.json", project.key()?)),
    )
}

fn read_receipt(ctx: &Context, project: &Project) -> Result<Option<Receipt>> {
    let path = receipt_path(ctx, project)?;
    match fs::symlink_metadata(&path) {
        Ok(info) if info.is_file() && info.len() <= 8192 => {
            Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Ok(_) => bail!("Invalid project verification receipt"),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn has_generated_receipt(ctx: &Context, path: &Path, stack: &str) -> bool {
    read_receipt(ctx, &Project::known(path, stack)).is_ok_and(|receipt| {
        receipt.is_some_and(|receipt| receipt.generated && receipt.stack == stack)
    })
}

fn record(ctx: &Context, project: &Project, receipt: &Receipt) -> Result<()> {
    let stage = templates::Stage::new(&records(ctx)?)?;
    let file = stage.0.join("receipt");
    fs::write(&file, serde_json::to_vec(receipt)?)?;
    fs::rename(file, receipt_path(ctx, project)?)?;
    Ok(())
}

fn ignored(stack: &str, name: &str) -> bool {
    matches!(name, ".git" | ".devline-infra" | ".DS_Store")
        || name.starts_with(".devkit-") && name != ".devkit-origin.json"
        || match stack {
            "rust" | "java" => name == "target",
            "node" => matches!(
                name,
                "node_modules"
                    | ".next"
                    | "dist"
                    | "coverage"
                    | "test-results"
                    | "playwright-report"
            ),
            "dotnet" => matches!(name, "obj" | "bin" | "TestResults"),
            "elixir" => matches!(name, "_build" | "deps"),
            "python" => matches!(
                name,
                ".venv" | "__pycache__" | ".pytest_cache" | ".mypy_cache"
            ),
            _ => false,
        }
}

#[derive(Default)]
struct Tree {
    count: u64,
    bytes: u64,
    source_only: bool,
    generated: bool,
}

impl Tree {
    fn visit(
        &mut self,
        stack: &str,
        root: &Path,
        path: &Path,
        copy: Option<&Path>,
        digest: &mut Sha256,
    ) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(path)?.collect::<io::Result<_>>()?;
        entries.sort_unstable_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let name = entry.file_name();
            let name = name.to_str().context("Project filename is not UTF-8")?;
            let input = entry.path();
            let meta = fs::symlink_metadata(&input)?;
            let relative = input.strip_prefix(root)?;
            if stack == "ruby" && relative == Path::new(".devline-offline/ruby-runtime") {
                continue;
            }
            if stack == "ruby" && relative == Path::new(MANAGED) && meta.is_dir() {
                let mut contents = fs::read_dir(&input)?;
                if contents
                    .next()
                    .transpose()?
                    .is_some_and(|entry| entry.file_name() == "ruby-runtime")
                    && contents.next().is_none()
                {
                    // An otherwise absent managed directory containing only
                    // derived runtime output must not change a generated proof.
                    continue;
                }
            }
            if stack == "swift" && relative == Path::new(".build") {
                continue;
            }
            if stack == "rust"
                && relative.parent() == Some(Path::new("gen/schemas"))
                && matches!(
                    name,
                    "acl-manifests.json"
                        | "capabilities.json"
                        | "desktop-schema.json"
                        | "macOS-schema.json"
                )
                && meta.is_file()
                && root.join("tauri.conf.json").is_file()
            {
                continue;
            }
            if stack == "rust"
                && matches!(relative.to_str(), Some("gen" | "gen/schemas"))
                && meta.is_dir()
                && root.join("tauri.conf.json").is_file()
            {
                if let Some(copy) = copy {
                    fs::create_dir(copy.join(relative))?;
                }
                // Track any user files here, but creation of the container for
                // Tauri's generated schemas alone does not change the source.
                self.visit(stack, root, &input, copy, digest)?;
                continue;
            }
            if stack == "elixir" {
                let mut components = relative.iter();
                if components.next().is_some_and(|name| name == "vendor")
                    && components.nth(1).is_some_and(|name| name == "_build")
                {
                    continue;
                }
            }
            if stack == "node"
                && [
                    "node_modules/.vite",
                    "node_modules/.vite-temp",
                    "node_modules/.vitest-cache",
                    "node_modules/.cache",
                ]
                .iter()
                .any(|cache| relative.starts_with(cache))
            {
                continue;
            }
            let in_managed = relative.starts_with(MANAGED);
            if self.source_only && (in_managed || stack == "go" && relative.starts_with("vendor")) {
                continue;
            }
            let bundled_node_modules =
                self.generated && stack == "node" && relative.starts_with("node_modules");
            let dependency_tree = in_managed
                || bundled_node_modules
                || ["vendor", "maven-repository", "nuget-packages", "wheelhouse"]
                    .iter()
                    .any(|directory| relative.starts_with(directory));
            if !dependency_tree && ignored(stack, name) {
                continue;
            }
            self.count += 1;
            self.bytes = self.bytes.saturating_add(meta.len());
            if self.count > 150_000 || self.bytes > 5 * 1024 * 1024 * 1024 {
                bail!("Project exceeds the offline verification size limit");
            }
            let name = relative.to_str().context("Project filename is not UTF-8")?;
            digest.update((name.len() as u64).to_le_bytes());
            digest.update(name.as_bytes());
            let output = copy.map(|copy| copy.join(relative));
            if meta.is_dir() {
                digest.update(b"d");
                if let Some(output) = output {
                    fs::create_dir(&output)?;
                }
                self.visit(stack, root, &input, copy, digest)?;
            } else if meta.is_file() {
                digest.update(b"f");
                digest.update(meta.len().to_le_bytes());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    digest.update((meta.permissions().mode() & 0o111).to_le_bytes());
                }
                let mut file = fs::File::open(&input)?;
                if let Some(output) = output {
                    let mut destination = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&output)?;
                    let bytes = io::copy(&mut file, &mut destination)?;
                    if bytes != meta.len() {
                        bail!("Project changed during preparation");
                    }
                    fs::set_permissions(&output, meta.permissions())?;
                    let mut file = fs::File::open(&output)?;
                    io::copy(&mut file, digest)?;
                } else {
                    let bytes = io::copy(&mut file, digest)?;
                    if bytes != meta.len() {
                        bail!("Project changed during preparation");
                    }
                }
            } else {
                bail!("Move symbolic links and special files out of the project before preparing: {name}");
            }
        }
        Ok(())
    }
}

fn proof_fingerprint(stack: &str, root: &Path, generated: bool) -> Result<String> {
    fingerprint_tree(stack, root, None, false, generated)
}

fn fingerprint_tree(
    stack: &str,
    root: &Path,
    copy: Option<&Path>,
    source_only: bool,
    generated: bool,
) -> Result<String> {
    let mut digest = Sha256::new();
    Tree {
        source_only,
        generated,
        ..Tree::default()
    }
    .visit(stack, root, root, copy, &mut digest)?;
    Ok(format!("{:x}", digest.finalize()))
}

/// Reuse the same safe source/dependency boundary for opt-in client mutations.
pub(crate) fn mutation_snapshot(stack: &str, root: &Path, copy: Option<&Path>) -> Result<String> {
    fingerprint_tree(stack, root, copy, false, false)
}

/// Publish a proof only when the original now exactly matches a tested copy.
pub(crate) fn record_mutation(ctx: &Context, destination: &Path, verified: &Path) -> Result<Value> {
    let project = Project::from_request(ctx, &json!({"path":destination}))?;
    let staged = Project::known(verified, &project.stack);
    let receipt = read_receipt(ctx, &staged)?
        .context("Client dependency mutation has no verification receipt")?;
    if receipt.generated
        || proof_fingerprint(&project.stack, destination, false)? != receipt.fingerprint
    {
        bail!("Project no longer matches the verified client dependency change");
    }
    record(ctx, &project, &receipt)?;
    Ok(report(
        &project,
        Some(&receipt),
        "ready_offline",
        "Selected client dependencies passed with external networking denied.",
    ))
}

fn report(project: &Project, receipt: Option<&Receipt>, status: &str, detail: &str) -> Value {
    json!({"path":project.path,"ready_offline":status=="ready_offline","status":status,"detail":detail,
        "adapter":(!project.stack.is_empty()).then_some(&project.stack),"fingerprint":receipt.map(|v|&v.fingerprint),
        "verified_at":receipt.map(|v|v.verified_at),"toolchain":receipt.map(|v|&v.toolchain),"toolchain_path":receipt.map(|v|&v.toolchain_path),
        "checks":[{"name":"Offline project test","ok":status=="ready_offline","detail":detail}],
        "run_command":"devline project run -- <command>"})
}

pub(crate) fn readiness(ctx: &Context, request: &Value) -> Result<Value> {
    let project = Project::from_request(ctx, request)?;
    readiness_for_project(ctx, project)
}

pub(crate) fn readiness_for_profile(ctx: &Context, profile: &Value) -> Result<Value> {
    readiness_for_project(ctx, Project::from_profile(profile)?)
}

fn readiness_for_project(ctx: &Context, project: Project) -> Result<Value> {
    if project.stack.is_empty() {
        return Ok(report(
            &project,
            None,
            "unsupported_project",
            &project.unsupported_detail(),
        ));
    }
    let Some(receipt) = read_receipt(ctx, &project)? else {
        return Ok(report(
            &project,
            None,
            "unverified",
            "Prepare and verify this project's dependencies on this Mac.",
        ));
    };
    let tool = project.toolchain(ctx);
    if receipt.stack != project.stack
        || !tool.is_ok_and(|(_, path, detail)| {
            path == receipt.toolchain_path && detail == receipt.toolchain
        })
    {
        return Ok(report(
            &project,
            Some(&receipt),
            "changed",
            "The project toolchain changed. Verify again with the installed toolchain.",
        ));
    }
    let current = match proof_fingerprint(&project.stack, &project.path, receipt.generated) {
        Ok(current) => current,
        Err(error) => {
            return Ok(report(
                &project,
                Some(&receipt),
                "changed",
                &format!("Could not verify current project files: {error:#}"),
            ))
        }
    };
    if current != receipt.fingerprint {
        return Ok(report(
            &project,
            Some(&receipt),
            "changed",
            "Project files or vendored dependencies changed. Verify again.",
        ));
    }
    Ok(report(
        &project,
        Some(&receipt),
        "ready_offline",
        "The current project passed with external networking denied.",
    ))
}

fn private_stage(parent: &Path) -> Result<templates::Stage> {
    let stage = templates::Stage::new(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&stage.0, fs::Permissions::from_mode(0o700))?;
    }
    Ok(stage)
}

fn step(job: &mut Option<&mut Job>, label: &str) -> Result<()> {
    if let Some(job) = job.as_deref_mut() {
        job.stage(label)?;
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Invocation {
    Prepare,
    Verify,
    GeneratedProject,
    Starter,
}

fn invoke(
    adapter: &Path,
    invocation: Invocation,
    project: &Path,
    tool: &Path,
    tool_name: &str,
    stage: &Path,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    let verify = invocation != Invocation::Prepare;
    let label = if verify {
        "Running isolated offline project tests"
    } else {
        "Resolving locked dependencies in a project copy"
    };
    let sandbox_path = serde_json::to_string(
        stage
            .to_str()
            .context("Project staging path is not UTF-8")?,
    )?;
    let network = if verify { NETWORK_POLICY } else { "" };
    let dotnet_mutex = if tool_name == "dotnet" {
        "(allow file-write* (subpath \"/private/tmp/.dotnet/shm\") (subpath \"/private/tmp/.dotnet/lockfiles\") (literal \"/private/tmp/.dotnet\"))"
    } else {
        ""
    };
    // Apple's atomic file writer uses the user's TemporaryItems directory even
    // when SwiftPM receives an isolated TMPDIR and scratch/cache directories.
    let swift_atomic = if tool_name == "swift" {
        let output = Command::new("/usr/bin/getconf")
            .arg("DARWIN_USER_TEMP_DIR")
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "Locate Swift build scratch directory"
        );
        let temp = Path::new(String::from_utf8_lossy(&output.stdout).trim()).canonicalize()?;
        let path = serde_json::to_string(&temp.join("TemporaryItems"))?;
        format!("(allow file-write* (subpath {path}))")
    } else {
        String::new()
    };
    let policy = format!("(version 1)(allow default)(deny file-write*)(allow file-write* (subpath {sandbox_path}) (literal \"/dev/null\")){dotnet_mutex}{swift_atomic}{network}");
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command.args(["-p", &policy, "/bin/sh"]);
    let home = stage.join(if verify {
        "verify-home"
    } else {
        "prepare-home"
    });
    let tmp = stage.join(if verify { "verify-tmp" } else { "prepare-tmp" });
    fs::create_dir(&home)?;
    fs::create_dir(&tmp)?;
    command
        .env_clear()
        .env("HOME", &home)
        .env("TMPDIR", &tmp)
        .env("DEVKIT_TOOLCHAIN", tool);
    if invocation == Invocation::Starter {
        templates::tool_environment(&mut command, tool_name, tool)?;
    } else {
        templates::project_tool_environment(&mut command, tool_name, tool)?;
    }
    if tool_name == "mvn" {
        command.env(
            "JDK_JAVA_OPTIONS",
            format!(
                "-Duser.home={} -Djava.io.tmpdir={}",
                serde_json::to_string(&home)?,
                serde_json::to_string(&tmp)?
            ),
        );
    }
    command.arg(adapter).current_dir(project);
    if matches!(
        invocation,
        Invocation::GeneratedProject | Invocation::Starter
    ) {
        for (key, path) in [
            ("GOCACHE", "go-build"),
            ("GOMODCACHE", "go-mod"),
            ("CARGO_HOME", "cargo"),
            ("CARGO_TARGET_DIR", "target"),
            ("DOTNET_CLI_HOME", "dotnet"),
            ("NUGET_PACKAGES", "nuget"),
            ("NUGET_HTTP_CACHE_PATH", "nuget-http"),
            ("MIX_HOME", "mix"),
            ("HEX_HOME", "hex"),
        ] {
            command.env(key, home.join(path));
        }
    } else {
        command
            .arg(if invocation == Invocation::Prepare {
                "prepare"
            } else {
                "verify"
            })
            .arg(project)
            .arg(project.join(MANAGED))
            .arg(tool);
    }
    templates::run_command(&mut command, label, job.as_deref_mut())
}

fn lock(ctx: &Context, project: &Project, job: &Option<&mut Job>) -> Result<fs::File> {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(records(ctx)?.join(format!("{}.lock", project.key()?)))?;
    loop {
        if let Some(job) = job.as_deref() {
            job.checkpoint()?;
        }
        match FileExt::try_lock_exclusive(&file) {
            Ok(()) => return Ok(file),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100))
            }
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn lock_generated(ctx: &Context, path: &Path, stack: &str) -> Result<fs::File> {
    lock(ctx, &Project::known(path, stack), &None)
}

fn publish(
    project: &Path,
    copy: &Path,
    stage: templates::Stage,
    outputs: &[&str],
    finish: impl FnOnce() -> Result<Receipt>,
) -> Result<Receipt> {
    let mut replaced = Vec::new();
    let result = (|| {
        for output in outputs {
            let source = copy.join(output);
            if !source.is_dir() || fs::symlink_metadata(&source)?.is_symlink() {
                bail!("Prepared dependency directory is missing: {output}");
            }
            let target = project.join(output);
            let backup = stage.0.join(format!("backup-{}", replaced.len()));
            let original = match fs::symlink_metadata(&target) {
                Ok(meta) if meta.is_dir() => {
                    fs::rename(&target, &backup)?;
                    Some(backup)
                }
                Ok(_) => bail!("Expected a directory at {}", target.display()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => return Err(error.into()),
            };
            replaced.push((target.clone(), source.clone(), original, false));
            templates::rename_exclusive(&source, &target)?;
            replaced.last_mut().expect("new publication record").3 = true;
        }
        finish()
    })();
    if result.is_err() {
        let rollback = (|| -> Result<()> {
            for (target, source, backup, published) in replaced.into_iter().rev() {
                if published {
                    templates::rename_exclusive(&target, &source)?;
                }
                if let Some(backup) = backup {
                    templates::rename_exclusive(&backup, &target)?;
                }
            }
            Ok(())
        })();
        if let Err(error) = rollback {
            let recovery = stage.0.clone();
            std::mem::forget(stage);
            return Err(error).with_context(|| {
                format!(
                    "Could not restore project dependency directories. Backups remain at {}",
                    recovery.display()
                )
            });
        }
    }
    result
}

pub(crate) fn lock_project(
    ctx: &Context,
    request: &Value,
    job: &Option<&mut Job>,
) -> Result<fs::File> {
    let path = Path::new(text_arg(request, "path")?).canonicalize()?;
    if !path.is_dir() {
        bail!("Choose an existing project directory");
    }
    lock(ctx, &Project::known(&path, ""), job)
}

pub(crate) fn prepare(ctx: &Context, request: &Value, job: Option<&mut Job>) -> Result<Value> {
    let _lock = lock_project(ctx, request, &job)?;
    prepare_locked(ctx, request, job)
}

// Caller holds the same project lock used by standalone dependency preparation.
pub(crate) fn prepare_locked(
    ctx: &Context,
    request: &Value,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    crate::service_clients::recover(ctx, &Path::new(text_arg(request, "path")?).canonicalize()?)?;
    let project = Project::from_request(ctx, request)?;
    if project.path.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a project outside the Devline library");
    }
    let offline = request["offline"] == true;
    let adapter = project.adapter(ctx)?;
    let (tool_name, tool, tool_detail) = project.toolchain(ctx)?;
    step(&mut job, "Copying project files for offline preparation")?;
    let stage = private_stage(&project.path)?;
    let copy = stage.0.join("project");
    fs::create_dir(&copy)?;
    let generated = offline
        && request["force_adapter"] != true
        && (!project.path.join(MANAGED).is_dir()
            || project.stack == "ruby"
                && !project.path.join(MANAGED).join("Gemfile.lock").is_file())
        && project::origin(&project.path).is_some()
        && fs::symlink_metadata(project.path.join("test-offline.sh"))
            .is_ok_and(|meta| meta.is_file());
    let original = fingerprint_tree(&project.stack, &project.path, Some(&copy), false, generated)?;
    let original_source = fingerprint_tree(&project.stack, &copy, None, true, false)?;
    if !offline {
        fs::create_dir_all(copy.join(MANAGED))?;
        invoke(
            &adapter,
            Invocation::Prepare,
            &copy,
            &tool,
            &tool_name,
            &stage.0,
            &mut job,
        )?;
    }
    if generated {
        invoke(
            &copy.join("test-offline.sh"),
            Invocation::GeneratedProject,
            &copy,
            &tool,
            &tool_name,
            &stage.0,
            &mut job,
        )?;
    } else {
        invoke(
            &adapter,
            Invocation::Verify,
            &copy,
            &tool,
            &tool_name,
            &stage.0,
            &mut job,
        )?;
    }
    if fingerprint_tree(&project.stack, &copy, None, true, false)? != original_source {
        bail!("Preparation changed source or lockfiles in the project copy. Review the adapter log and update the project lockfiles before retrying.");
    }
    let tested = proof_fingerprint(&project.stack, &copy, generated)?;
    if offline && tested != original {
        bail!("Verification changed project dependencies in the isolated copy. Prepare the project before checking it offline.");
    }
    step(&mut job, "Checking project files before publication")?;
    if proof_fingerprint(&project.stack, &project.path, generated)? != original {
        bail!("Project files changed during preparation. Run preparation again.");
    }
    let finish = || -> Result<Receipt> {
        let fingerprint = proof_fingerprint(&project.stack, &project.path, generated)?;
        if fingerprint != tested {
            bail!("Project dependencies changed before verification could be saved");
        }
        let receipt = Receipt {
            fingerprint,
            stack: project.stack.clone(),
            toolchain: tool_detail,
            toolchain_path: tool,
            verified_at: crate::tasks::now(),
            generated,
        };
        record(ctx, &project, &receipt)?;
        Ok(receipt)
    };
    let receipt = if offline {
        finish()?
    } else {
        let outputs = if project.stack == "go" && copy.join("vendor").is_dir() {
            &[MANAGED, "vendor"][..]
        } else {
            &[MANAGED][..]
        };
        publish(&project.path, &copy, stage, outputs, finish)?
    };
    Ok(report(
        &project,
        Some(&receipt),
        "ready_offline",
        "The current project passed with external networking denied.",
    ))
}

pub(crate) fn runner(ctx: &Context, request: &Value) -> Result<Value> {
    let project = Project::from_request(ctx, request)?;
    let adapter = project.adapter(ctx)?;
    let (_, tool, _) = project.toolchain(ctx)?;
    Ok(
        json!({"path":project.path,"adapter":adapter,"cache":project.path.join(MANAGED),"toolchain_path":tool}),
    )
}

pub(crate) fn create(ctx: &Context, request: &Value) -> Result<Value> {
    create_with_job(ctx, request, None)
}

pub(crate) fn create_with_job(
    ctx: &Context,
    request: &Value,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    if let Some(job) = job.as_deref_mut() {
        job.stage("Planning the project and selected services")?;
    }
    let id = text_arg(request, "id")?;
    let destination = Path::new(text_arg(request, "destination")?);
    let mut plan = templates::create(ctx, id, destination, true)?;
    let exact = format!(
        "{}@{}",
        text_arg(&plan, "template")?,
        text_arg(&plan, "version")?
    );
    let path = PathBuf::from(text_arg(&plan, "path")?);
    let mut selected = std::collections::BTreeMap::new();
    for service in plan["services"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(request["services"].as_array().into_iter().flatten())
    {
        let reference = service
            .as_str()
            .context("Invalid starter service release")?;
        let (name, version) = reference
            .split_once('@')
            .context("Choose exact infrastructure releases as SERVICE@VERSION")?;
        if selected
            .insert(name, version)
            .is_some_and(|previous| previous != version)
        {
            bail!("Choose one version of {name}. This starter has a required release.");
        }
    }
    let services: Vec<_> = selected
        .into_iter()
        .map(|(id, version)| format!("{id}@{version}"))
        .collect();
    let service_plan = crate::infrastructure::plan_project(ctx, &services, &request["ports"])?;
    let compatible_docker = service_plan.is_null()
        || service_plan["docker"]["available"] == true
            && service_plan["docker"]["platform"] == service_plan["platform"];
    let offline = request["offline"] == true;
    if request["dry_run"] == true {
        let images = service_plan["images"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        plan["can_create"] = json!(
            compatible_docker && (!offline || images.iter().all(|image| image["archived"] == true))
        );
        plan["services"] = json!(services);
        plan["service_plan"] = service_plan;
        plan["offline"] = json!(offline);
        if request["clients"]
            .as_array()
            .is_some_and(|clients| !clients.is_empty())
        {
            let clients = crate::service_clients::catalog(ctx, &json!({"id":exact}))?;
            for chosen in request["clients"].as_array().unwrap() {
                let client = clients["services"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .flat_map(|service| service["libraries"].as_array().into_iter().flatten())
                    .find(|client| client["id"] == *chosen)
                    .context("Client selection does not match the selected starter runtime")?;
                if offline && client["installed"] != true && client["offline_available"] != true {
                    plan["can_create"] = json!(false);
                }
            }
            plan["client_dependencies"] = clients;
        }
        return Ok(plan);
    }
    if !compatible_docker {
        if let Some(detail) = service_plan["docker"]["detail"].as_str() {
            bail!("Start a compatible local Docker engine before creating this project. {detail}");
        }
        bail!(
            "Start a local Docker engine with {} support before creating this project",
            service_plan["platform"]
                .as_str()
                .unwrap_or("the required platform")
        );
    }
    for service in &services {
        let verify = json!({"command":"infra.verify","id":service});
        if let Err(error) = crate::infrastructure::execute(ctx, &verify, job.as_deref_mut()) {
            if offline {
                return Err(error).with_context(||format!("Archive {service} before creating this project. Prepare it online with devline infra prepare {service}, or import a vault bundle"));
            }
            crate::infrastructure::execute(
                ctx,
                &json!({"command":"infra.prepare","id":service}),
                job.as_deref_mut(),
            )?;
        }
    }
    if let Some(job) = job.as_deref_mut() {
        job.stage("Installing the starter and its documentation")?;
    }
    templates::install(ctx, &exact)?;
    for doc in plan["docs"].as_array().into_iter().flatten() {
        let doc = doc
            .as_str()
            .context("Invalid starter documentation release")?;
        let (id, version) = doc
            .split_once('@')
            .map_or((doc, None), |(id, version)| (id, Some(version)));
        crate::install_docs_job(ctx, id, version, job.as_deref_mut())?;
    }
    let mut result = templates::create_with_job(ctx, &exact, &path, false, job.as_deref_mut())?;
    if request["clients"]
        .as_array()
        .is_some_and(|clients| !clients.is_empty())
    {
        match crate::service_clients::add(
            ctx,
            &json!({"path":path,"clients":request["clients"],"offline":offline}),
            job.as_deref_mut(),
        ) {
            Ok(clients) => result["client_dependencies"] = clients,
            Err(error) => {
                let removed = rollback_created(ctx, &path, text_arg(&result, "creation_digest")?)?;
                return Err(error).with_context(|| if removed {
                    "Could not prepare the explicitly selected client dependencies; the new project was removed".to_owned()
                } else {
                    format!("Could not prepare the explicitly selected client dependencies. Files changed during creation and were preserved at {}", path.display())
                });
            }
        }
    }
    if !services.is_empty() {
        let configured = crate::infrastructure::execute(
            ctx,
            &json!({"command":"infra.stack.create","path":path,"services":services,"ports":request["ports"]}),
            job,
        );
        match configured {
            Ok(configured) => result["infrastructure"] = configured,
            Err(error) => {
                let removed = rollback_created(ctx, &path, text_arg(&result, "creation_digest")?)?;
                return Err(error).with_context(|| if removed {
                    "Could not configure the starter's required infrastructure; the new project was removed".to_owned()
                } else {
                    format!("Could not configure the starter's required infrastructure. Files changed during creation and were preserved at {}", path.display())
                });
            }
        }
    }
    result["ready_offline"] = json!(true);
    result["services"] = json!(services);
    Ok(result)
}

fn rollback_created(ctx: &Context, path: &Path, creation_digest: &str) -> Result<bool> {
    let _lock = lock_project(ctx, &json!({"path":path}), &None)?;
    if !templates::complete_tree_digest(path).is_ok_and(|digest| digest == creation_digest) {
        return Ok(false);
    }
    let rollback = private_stage(path.parent().context("Project destination has no parent")?)?;
    let moved = rollback.0.join("project");
    templates::rename_exclusive(path, &moved)?;
    // Check again after the atomic move to catch edits made between the first
    // snapshot and publication into rollback staging. Never delete those edits.
    if !templates::complete_tree_digest(&moved).is_ok_and(|digest| digest == creation_digest) {
        if let Err(error) = templates::rename_exclusive(&moved, path) {
            std::mem::forget(rollback);
            return Err(error).with_context(|| {
                format!(
                    "Project changed during creation rollback; all files were preserved at {}",
                    moved.display()
                )
            });
        }
        return Ok(false);
    }
    let _ = fs::remove_file(receipt_path(ctx, &Project::known(path, ""))?);
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn verify_generated(
    ctx: &Context,
    source: &Path,
    destination: &Path,
    stack: &str,
    tool_name: &str,
    tool: &Path,
    detail: &str,
    mut job: Option<&mut Job>,
) -> Result<()> {
    let project = Project::known(destination, stack);
    let stage = private_stage(&records(ctx)?)?;
    let copy = stage.0.join("project");
    fs::create_dir(&copy)?;
    let tested = fingerprint_tree(stack, source, Some(&copy), false, true)?;
    let script = copy.join("test-offline.sh");
    if !fs::symlink_metadata(&script).is_ok_and(|meta| meta.is_file()) {
        bail!("An offline verification script is required");
    }
    invoke(
        &script,
        Invocation::Starter,
        &copy,
        tool,
        tool_name,
        &stage.0,
        &mut job,
    )?;
    if proof_fingerprint(stack, &copy, true)? != tested {
        bail!("The starter test changed source files or vendored dependencies; the original project was not verified");
    }
    if proof_fingerprint(stack, source, true)? != tested {
        bail!("Project source changed before verification could be saved");
    }
    let receipt = Receipt {
        fingerprint: tested,
        stack: stack.into(),
        toolchain: detail.into(),
        toolchain_path: tool.into(),
        verified_at: crate::tasks::now(),
        generated: true,
    };
    record(ctx, &project, &receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn starter_test_cannot_repair_source_or_dependencies_only_in_its_copy() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir().canonicalize()?)?;
        let home = root.0.join("home");
        let source = root.0.join("source");
        fs::create_dir(&home)?;
        fs::create_dir_all(source.join("vendor"))?;
        fs::write(source.join("main.rs"), "original source")?;
        fs::write(source.join("vendor/client.rs"), "original dependency")?;
        let ctx = Context {
            home,
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        for (index, changed) in ["main.rs", "vendor/client.rs"].iter().enumerate() {
            fs::write(
                source.join("test-offline.sh"),
                format!("#!/bin/sh\nprintf repaired > {changed}\nexit 0\n"),
            )?;
            let destination = root.0.join(format!("project-{index}"));
            let error = verify_generated(
                &ctx,
                &source,
                &destination,
                "rust",
                "rustc",
                Path::new("/bin/sh"),
                "fixture",
                None,
            )
            .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("test changed source files or vendored dependencies"),
                "{error:#}"
            );
            assert!(read_receipt(&ctx, &Project::known(&destination, "rust"))?.is_none());
            assert!(!destination.exists());
        }
        assert_eq!(
            fs::read_to_string(source.join("main.rs"))?,
            "original source"
        );
        assert_eq!(
            fs::read_to_string(source.join("vendor/client.rs"))?,
            "original dependency"
        );
        // Real build output is still permitted and excluded from source proof.
        fs::write(
            source.join("test-offline.sh"),
            "#!/bin/sh\nmkdir target\nprintf built > target/output\n",
        )?;
        let destination = root.0.join("valid-project");
        verify_generated(
            &ctx,
            &source,
            &destination,
            "rust",
            "rustc",
            Path::new("/bin/sh"),
            "fixture",
            None,
        )?;
        assert!(read_receipt(&ctx, &Project::known(&destination, "rust"))?.is_some());
        Ok(())
    }

    #[test]
    fn creation_rollback_preserves_new_git_history_and_ignored_files() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir().canonicalize()?)?;
        let home = root.0.join("home");
        fs::create_dir(&home)?;
        let ctx = Context {
            home,
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        for (index, extra) in [
            ".git/objects/new-history",
            "target/user-notes",
            ".devline-offline/ruby-runtime/custom",
            "installed.json",
        ]
        .iter()
        .enumerate()
        {
            let staging = templates::Stage::new(&root.0)?;
            fs::write(staging.0.join("main.rs"), "fn main() {}")?;
            let digest = templates::complete_tree_digest(&staging.0)?;
            let destination = root.0.join(format!("project-{index}"));
            staging.publish(&destination)?;
            let extra = destination.join(extra);
            fs::create_dir_all(extra.parent().unwrap())?;
            fs::write(&extra, "new user data")?;
            assert!(!rollback_created(&ctx, &destination, &digest)?);
            assert_eq!(fs::read_to_string(extra)?, "new user data");
            assert!(destination.join("main.rs").is_file());
        }
        let staging = templates::Stage::new(&root.0)?;
        fs::write(staging.0.join("main.rs"), "fn main() {}")?;
        let digest = templates::complete_tree_digest(&staging.0)?;
        let destination = root.0.join("unchanged");
        staging.publish(&destination)?;
        assert!(rollback_created(&ctx, &destination, &digest)?);
        assert!(!destination.exists());
        Ok(())
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn compiled_ruby_runtime_cache_does_not_change_project_proof() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir())?;
        fs::write(root.0.join("Gemfile"), "source 'https://rubygems.org'\n")?;
        let before = proof_fingerprint("ruby", &root.0, true)?;
        fs::create_dir_all(root.0.join(".devline-offline/ruby-runtime/key"))?;
        fs::write(
            root.0
                .join(".devline-offline/ruby-runtime/key/compiled.bundle"),
            "derived native output",
        )?;
        assert_eq!(before, proof_fingerprint("ruby", &root.0, true)?);
        fs::write(
            root.0.join(".devline-offline/Gemfile.lock"),
            "authoritative lock",
        )?;
        assert_ne!(before, proof_fingerprint("ruby", &root.0, true)?);
        Ok(())
    }

    #[test]
    fn starter_verification_and_existing_projects_keep_distinct_java_environments() -> Result<()> {
        const CHILD: &str = "DEVLINE_JAVA_ENV_REGRESSION_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = Command::new(std::env::current_exe()?)
                .args(["--exact", "project_prepare::tests::starter_verification_and_existing_projects_keep_distinct_java_environments", "--nocapture"])
                .env(CHILD, "1")
                .env("JAVA_HOME", "/devline-test-active-project-jdk")
                .output()?;
            anyhow::ensure!(
                output.status.success(),
                "{} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            anyhow::ensure!(
                String::from_utf8_lossy(&output.stdout).contains("1 passed"),
                "The Java environment regression child did not run"
            );
            return Ok(());
        }
        let root = private_stage(&std::env::temp_dir().canonicalize()?)?;
        let home = root.0.join("library");
        let source = root.0.join("source");
        fs::create_dir(&home)?;
        fs::create_dir(&source)?;
        let ctx = Context {
            home,
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: Vec::new(),
                templates: Vec::new(),
            },
        };
        let jdk = Command::new("/usr/libexec/java_home")
            .args(["-F", "-v", "26"])
            .output()?;
        let expected = if jdk.status.success() {
            String::from_utf8_lossy(&jdk.stdout).trim().to_owned()
        } else {
            String::new()
        };
        let script = source.join("test-offline.sh");
        let quoted = |value: &str| format!("'{}'", value.replace('\'', "'\"'\"'"));
        fs::write(
            &script,
            format!(
                "#!/bin/sh\ntest \"${{JAVA_HOME:-}}\" = {}\n",
                quoted(&expected)
            ),
        )?;
        // Exercise the actual starter publication verifier, not just the environment
        // helper. It must match the JDK used when the starter toolchain was probed.
        verify_generated(
            &ctx,
            &source,
            &root.0.join("destination"),
            "java",
            "mvn",
            Path::new("/bin/sh"),
            "fixture Maven toolchain",
            None,
        )?;
        fs::write(
            &script,
            "#!/bin/sh\ntest \"${JAVA_HOME:-}\" = /devline-test-active-project-jdk\n",
        )?;
        let stage = private_stage(&root.0)?;
        invoke(
            &script,
            Invocation::GeneratedProject,
            &source,
            Path::new("/bin/sh"),
            "mvn",
            &stage.0,
            &mut None,
        )?;
        Ok(())
    }

    #[test]
    fn desktop_build_outputs_do_not_hide_source_changes() -> Result<()> {
        let stage = private_stage(&std::env::temp_dir().canonicalize()?)?;
        fs::write(stage.0.join("Package.swift"), "// swift-tools-version: 6.0")?;
        let swift = proof_fingerprint("swift", &stage.0, true)?;
        fs::create_dir(stage.0.join(".build"))?;
        fs::write(stage.0.join(".build/output"), "generated")?;
        assert_eq!(swift, proof_fingerprint("swift", &stage.0, true)?);
        fs::create_dir_all(stage.0.join(".swiftpm/configuration"))?;
        fs::write(
            stage.0.join(".swiftpm/configuration/mirrors.json"),
            "changed",
        )?;
        assert_ne!(swift, proof_fingerprint("swift", &stage.0, true)?);

        fs::write(stage.0.join("tauri.conf.json"), "{}")?;
        let rust = proof_fingerprint("rust", &stage.0, true)?;
        fs::create_dir_all(stage.0.join("gen/schemas"))?;
        fs::write(stage.0.join("gen/schemas/desktop-schema.json"), "generated")?;
        assert_eq!(rust, proof_fingerprint("rust", &stage.0, true)?);
        fs::write(stage.0.join("gen/schemas/user.json"), "user schema")?;
        assert_ne!(rust, proof_fingerprint("rust", &stage.0, true)?);
        fs::remove_file(stage.0.join("gen/schemas/user.json"))?;
        fs::write(stage.0.join("gen/user.rs"), "user source")?;
        assert_ne!(rust, proof_fingerprint("rust", &stage.0, true)?);
        Ok(())
    }

    #[test]
    fn trusted_manifestless_origin_does_not_mask_a_real_manifest() -> Result<()> {
        let root = private_stage(&std::env::temp_dir().canonicalize()?)?;
        let home = root.0.join("library");
        let outer = root.0.join("go-project");
        let cli = outer.join("ruby-cli");
        fs::create_dir(&home)?;
        fs::create_dir_all(&cli)?;
        fs::write(outer.join("go.mod"), "module example.com/outer\n")?;
        fs::write(
            cli.join(".devkit-origin.json"),
            serde_json::to_vec(
                &json!({"template":"ruby/json-cli","version":"1.0.0","digest":"a".repeat(64)}),
            )?,
        )?;
        let ctx = Context {
            home,
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: Vec::new(),
                templates: Vec::new(),
            },
        };
        assert_eq!(project::context(&ctx, &cli)?["stack"], "go");
        record(
            &ctx,
            &Project::known(&cli, "ruby"),
            &Receipt {
                fingerprint: String::new(),
                stack: "ruby".into(),
                toolchain: "ruby".into(),
                toolchain_path: "ruby".into(),
                verified_at: 0,
                generated: true,
            },
        )?;
        let generated = project::context(&ctx, &cli)?;
        assert_eq!(generated["stack"], "ruby");
        assert_eq!(generated["path"], json!(cli));
        fs::write(cli.join("package.json"), "{}")?;
        assert_eq!(project::context(&ctx, &cli)?["stack"], "node");
        Ok(())
    }

    #[test]
    fn elixir_proof_ignores_rebar_outputs_and_tracks_vendored_source() -> Result<()> {
        let stage = private_stage(&std::env::temp_dir().canonicalize()?)?;
        let package = stage.0.join("vendor/ranch");
        fs::create_dir_all(package.join("src"))?;
        let source = package.join("src/ranch.erl");
        fs::write(&source, "before")?;
        let proof = proof_fingerprint("elixir", &stage.0, true)?;
        fs::create_dir_all(package.join("_build/default"))?;
        fs::write(package.join("_build/default/source.dag"), "build output")?;
        assert_eq!(proof, proof_fingerprint("elixir", &stage.0, true)?);
        fs::write(source, "after")?;
        assert_ne!(proof, proof_fingerprint("elixir", &stage.0, true)?);
        Ok(())
    }

    #[test]
    fn generated_node_proof_tracks_bundled_dependencies_and_ignores_build_output() -> Result<()> {
        let stage = private_stage(&std::env::temp_dir().canonicalize()?)?;
        fs::write(stage.0.join("package.json"), "{}")?;
        fs::create_dir_all(stage.0.join("node_modules/library"))?;
        let dependency = stage.0.join("node_modules/library/index.js");
        fs::write(&dependency, "initial")?;
        let proof = proof_fingerprint("node", &stage.0, true)?;
        let unbundled_proof = proof_fingerprint("node", &stage.0, false)?;
        fs::create_dir(stage.0.join("dist"))?;
        fs::write(stage.0.join("dist/index.html"), "build artifact")?;
        assert_eq!(proof, proof_fingerprint("node", &stage.0, true)?);
        fs::remove_file(dependency)?;
        assert_ne!(proof, proof_fingerprint("node", &stage.0, true)?);
        assert_eq!(unbundled_proof, proof_fingerprint("node", &stage.0, false)?);
        Ok(())
    }

    #[test]
    fn rust_snapshot_keeps_vendored_target_sources_and_excludes_build_target() -> Result<()> {
        let root = private_stage(&std::env::temp_dir().canonicalize()?)?;
        let source = root.0.join("source");
        let copy = root.0.join("copy");
        fs::create_dir_all(source.join("vendor/cc/src/target"))?;
        fs::create_dir_all(source.join("target/debug"))?;
        fs::create_dir(&copy)?;
        fs::write(source.join("vendor/cc/src/target/llvm.rs"), "source")?;
        fs::write(source.join("target/debug/build-artifact"), "output")?;
        let proof = fingerprint_tree("rust", &source, Some(&copy), false, true)?;
        assert_eq!(
            fs::read_to_string(copy.join("vendor/cc/src/target/llvm.rs"))?,
            "source"
        );
        assert!(!copy.join("target").exists());
        assert_eq!(proof, proof_fingerprint("rust", &copy, true)?);
        Ok(())
    }

    #[test]
    fn failed_publication_restores_original_vendor_and_managed_cache() -> Result<()> {
        let root = private_stage(&std::env::temp_dir().canonicalize()?)?;
        let project = root.0.join("project");
        fs::create_dir(&project)?;
        let work = private_stage(&project)?;
        let copy = work.0.join("copy");
        for path in [MANAGED, "vendor"] {
            fs::create_dir_all(project.join(path))?;
            fs::write(project.join(path).join("evidence"), "original")?;
            fs::create_dir_all(copy.join(path))?;
            fs::write(copy.join(path).join("evidence"), "prepared")?;
        }
        let result = publish(&project, &copy, work, &[MANAGED, "vendor"], || {
            bail!("injected receipt failure")
        });
        assert!(result.is_err());
        for path in [MANAGED, "vendor"] {
            assert_eq!(
                fs::read_to_string(project.join(path).join("evidence"))?,
                "original"
            );
        }
        Ok(())
    }
}
