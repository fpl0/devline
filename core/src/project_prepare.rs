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
        let reports = templates::runtime_reports(ctx, std::slice::from_ref(&self.stack))?;
        let report = reports
            .as_array()
            .and_then(|reports| reports.first())
            .context("No toolchain report is available")?;
        let detail = text_arg(report, "detail")?;
        let path = report["path"]
            .as_str()
            .filter(|_| report["available"] == true)
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

fn invoke(
    adapter: &Path,
    action: &str,
    project: &Path,
    tool: &Path,
    tool_name: &str,
    stage: &Path,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    let verify = action != "prepare";
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
    let policy = format!("(version 1)(allow default)(deny file-write*)(allow file-write* (subpath {sandbox_path}) (literal \"/dev/null\")){dotnet_mutex}{network}");
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
    templates::tool_environment(&mut command, tool_name, tool)?;
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
    if action == "generated" {
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
            .arg(action)
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

pub(crate) fn prepare(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let project = Project::from_request(ctx, request)?;
    if project.path.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a project outside the Devline library");
    }
    let offline = request["offline"] == true;
    let adapter = project.adapter(ctx)?;
    let (tool_name, tool, tool_detail) = project.toolchain(ctx)?;
    let _lock = lock(ctx, &project, &job)?;
    step(&mut job, "Copying project files for offline preparation")?;
    let stage = private_stage(&project.path)?;
    let copy = stage.0.join("project");
    fs::create_dir(&copy)?;
    let generated = offline
        && project::origin(&project.path).is_some()
        && fs::symlink_metadata(project.path.join("test-offline.sh"))
            .is_ok_and(|meta| meta.is_file());
    let original = fingerprint_tree(&project.stack, &project.path, Some(&copy), false, generated)?;
    let original_source = fingerprint_tree(&project.stack, &copy, None, true, false)?;
    if !offline {
        fs::create_dir_all(copy.join(MANAGED))?;
        invoke(
            &adapter, "prepare", &copy, &tool, &tool_name, &stage.0, &mut job,
        )?;
    }
    if generated {
        invoke(
            &copy.join("test-offline.sh"),
            "generated",
            &copy,
            &tool,
            &tool_name,
            &stage.0,
            &mut job,
        )?;
    } else {
        invoke(
            &adapter, "verify", &copy, &tool, &tool_name, &stage.0, &mut job,
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
        if let Err(error) = crate::infrastructure::execute(ctx, &verify, None) {
            if offline {
                return Err(error).with_context(||format!("Archive {service} before creating this project. Prepare it online with devline infra prepare {service}, or import a vault bundle"));
            }
            crate::infrastructure::execute(
                ctx,
                &json!({"command":"infra.prepare","id":service}),
                None,
            )?;
        }
    }
    templates::install(ctx, &exact)?;
    for doc in plan["docs"].as_array().into_iter().flatten() {
        let doc = doc
            .as_str()
            .context("Invalid starter documentation release")?;
        let (id, version) = doc
            .split_once('@')
            .map_or((doc, None), |(id, version)| (id, Some(version)));
        crate::install_docs(ctx, id, version)?;
    }
    let mut result = templates::create(ctx, &exact, &path, false)?;
    if !services.is_empty() {
        let configured = crate::infrastructure::execute(
            ctx,
            &json!({"command":"infra.stack.create","path":path,"services":services,"ports":request["ports"]}),
            None,
        );
        match configured {
            Ok(configured) => result["infrastructure"] = configured,
            Err(error) => {
                let project = Project::from_request(ctx, &json!({"path":path}))?;
                if read_receipt(ctx, &project)?.is_some_and(|receipt| {
                    proof_fingerprint(&project.stack, &path, receipt.generated)
                        .is_ok_and(|hash| hash == receipt.fingerprint)
                }) {
                    let rollback =
                        private_stage(path.parent().context("Project destination has no parent")?)?;
                    templates::rename_exclusive(&path, &rollback.0.join("project"))?;
                    let _ = fs::remove_file(receipt_path(ctx, &project)?);
                }
                return Err(error)
                    .context("Could not configure the starter's required infrastructure");
            }
        }
    }
    result["ready_offline"] = json!(true);
    result["services"] = json!(services);
    Ok(result)
}

pub(crate) fn verify_generated(
    ctx: &Context,
    source: &Path,
    destination: &Path,
    stack: &str,
    tool_name: &str,
    tool: &Path,
    detail: &str,
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
        "generated",
        &copy,
        tool,
        tool_name,
        &stage.0,
        &mut None,
    )?;
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
