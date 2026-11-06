use crate::{lock_library, Context, Template};
use anyhow::{bail, Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const NAME_TOKEN: &str = "__PROJECT_NAME__";
static STAGE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
struct Installed {
    template: Template,
    digest: String,
}

#[derive(Clone)]
struct Entry {
    template: Template,
    installed: Option<PathBuf>,
}

fn slug(value: &str) -> bool {
    value.len() <= 64
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.ends_with('-')
}

fn version(value: &str) -> Result<[u64; 3]> {
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || (part.len() > 1 && part.starts_with('0'))
                || !part.bytes().all(|b| b.is_ascii_digit())
        })
    {
        bail!("Expected a template version in major.minor.patch format");
    }
    Ok([parts[0].parse()?, parts[1].parse()?, parts[2].parse()?])
}

fn id_parts(id: &str) -> Result<(&str, &str)> {
    let (stack, name) = id
        .split_once('/')
        .context("Expected a template ID such as go/chi-api")?;
    if !slug(stack) || !slug(name) {
        bail!("Invalid template ID {id}");
    }
    Ok((stack, name))
}

fn directory(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)
        .with_context(|| format!("Missing directory {}", path.display()))?
        .is_dir()
    {
        bail!("Expected a real directory at {}", path.display());
    }
    Ok(())
}

fn regular(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.is_file() {
        bail!("Expected a regular file at {}", path.display());
    }
    Ok(())
}

fn library(ctx: &Context, path: &Path, create: bool) -> Result<PathBuf> {
    let mut result = ctx.home.canonicalize()?;
    for part in path.components() {
        let Component::Normal(part) = part else {
            bail!("Invalid library path")
        };
        result.push(part);
        if create {
            match fs::create_dir(&result) {
                Ok(()) => (),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error.into()),
            }
        }
        match fs::symlink_metadata(&result) {
            Ok(metadata) if !metadata.is_dir() => {
                bail!("Library path is not a directory: {}", result.display())
            }
            Ok(_) => (),
            Err(error) if !create && error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(result)
}

fn installed_base(ctx: &Context, id: &str, create: bool) -> Result<PathBuf> {
    let (stack, name) = id_parts(id)?;
    library(ctx, &Path::new("templates").join(stack).join(name), create)
}

fn read_marker(path: &Path) -> Result<Installed> {
    directory(path)?;
    directory(&path.join("project"))?;
    let marker = path.join("installed.json");
    regular(&marker)?;
    Ok(serde_json::from_slice(&fs::read(marker)?)?)
}

fn catalog_entries(ctx: &Context) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for template in &ctx.catalog.templates {
        id_parts(&template.id)?;
        version(&template.version)?;
        ids.insert(template.id.clone());
        entries.push(Entry {
            template: template.clone(),
            installed: None,
        });
    }
    for id in ids {
        let base = installed_base(ctx, &id, false)?;
        let dirs = match fs::read_dir(base) {
            Ok(dirs) => dirs,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        for dir in dirs {
            let dir = dir?;
            let label = dir.file_name();
            let Some(label) = label.to_str() else {
                continue;
            };
            if version(label).is_err() {
                continue;
            }
            let marker = read_marker(&dir.path())?;
            if marker.template.id != id || marker.template.version != label {
                bail!("Installed template marker does not match {}@{}", id, label);
            }
            if let Some(entry) = entries
                .iter_mut()
                .find(|entry| entry.template.id == id && entry.template.version == label)
            {
                entry.installed = Some(dir.path());
            } else {
                entries.push(Entry {
                    template: marker.template,
                    installed: Some(dir.path()),
                });
            }
        }
    }
    entries.sort_by(|a, b| {
        a.template.id.cmp(&b.template.id).then_with(|| {
            version(&b.template.version)
                .unwrap()
                .cmp(&version(&a.template.version).unwrap())
        })
    });
    Ok(entries)
}

fn select(ctx: &Context, reference: &str, prefer_installed: bool) -> Result<Entry> {
    let (id, pin) = match reference.split_once('@') {
        Some((id, pin)) => {
            version(pin)?;
            (id, Some(pin))
        }
        None => (reference, None),
    };
    id_parts(id)?;
    let entries = catalog_entries(ctx)?;
    let mut choices = entries
        .into_iter()
        .filter(|entry| entry.template.id == id && pin.is_none_or(|v| entry.template.version == v));
    if prefer_installed && pin.is_none() {
        let mut choices = choices.clone();
        if let Some(entry) = choices
            .find(|e| e.installed.is_some() && e.template.verified && target_matches(&e.template))
        {
            return Ok(entry);
        }
    }
    choices
        .next()
        .with_context(|| format!("No template release matches {reference}"))
}

fn source(ctx: &Context, path: &str) -> Result<PathBuf> {
    let mut result = ctx.catalog_dir.canonicalize()?;
    for part in Path::new(path).components() {
        let Component::Normal(part) = part else {
            bail!("Invalid catalog template path")
        };
        result.push(part);
        directory(&result)?;
    }
    Ok(result)
}

struct Stage(PathBuf);

impl Stage {
    fn new(parent: &Path) -> Result<Self> {
        for _ in 0..100 {
            let id = STAGE_ID.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".devkit-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e.into()),
            }
        }
        bail!("Could not reserve a staging directory")
    }

    fn publish(&self, destination: &Path) -> Result<()> {
        rename_exclusive(&self.0, destination)
            .with_context(|| format!("Could not create {}", destination.display()))
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(target_os = "macos")]
fn rename_exclusive(from: &Path, to: &Path) -> io::Result<()> {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_uint};
    use std::os::unix::ffi::OsStrExt;
    extern "C" {
        fn renamex_np(from: *const c_char, to: *const c_char, flags: c_uint) -> c_int;
    }
    const RENAME_EXCL: c_uint = 4;
    let from = CString::new(from.as_os_str().as_bytes())?;
    let to = CString::new(to.as_os_str().as_bytes())?;
    if unsafe { renamex_np(from.as_ptr(), to.as_ptr(), RENAME_EXCL) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(target_os = "macos"))]
fn rename_exclusive(from: &Path, to: &Path) -> io::Result<()> {
    match fs::symlink_metadata(to) {
        Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists)),
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        Err(_) => (),
    }
    fs::rename(from, to)
}

fn executable(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        true
    }
}

fn copy_tree(from: &Path, to: &Path, name: Option<&str>) -> Result<()> {
    directory(from)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        let target = to.join(entry.file_name());
        if metadata.is_dir() {
            fs::create_dir(&target)?;
            let dependency = matches!(
                entry.file_name().to_str(),
                Some("vendor" | "wheelhouse" | "nuget-packages" | "maven-repository" | "hex-cache")
            );
            copy_tree(&entry.path(), &target, if dependency { None } else { name })?;
        } else if metadata.is_file() {
            if let Some(name) = name.filter(|_| metadata.len() < 1024 * 1024) {
                let contents = fs::read(entry.path())?;
                if let Ok(text) = std::str::from_utf8(&contents) {
                    fs::write(&target, text.replace(NAME_TOKEN, name))?;
                } else {
                    fs::write(&target, contents)?;
                }
            } else {
                fs::copy(entry.path(), &target)?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &target,
                    fs::Permissions::from_mode(if executable(&metadata) { 0o755 } else { 0o644 }),
                )?;
            }
        } else {
            bail!(
                "Template entry is not a regular file or directory: {}",
                entry.path().display()
            );
        }
    }
    Ok(())
}

fn tree_digest(root: &Path) -> Result<String> {
    fn walk(root: &Path, relative: &Path, hash: &mut Sha256) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(root.join(relative))?.collect::<io::Result<_>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            if relative.as_os_str().is_empty() && entry.file_name() == "installed.json" {
                continue;
            }
            let path = relative.join(entry.file_name());
            let metadata = fs::symlink_metadata(entry.path())?;
            let name = path
                .to_str()
                .context("Template filename is not valid UTF-8")?;
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name);
            if metadata.is_dir() {
                hash.update([0]);
                walk(root, &path, hash)?;
            } else if metadata.is_file() {
                hash.update([1, u8::from(executable(&metadata))]);
                hash.update(metadata.len().to_le_bytes());
                let mut file = fs::File::open(entry.path())?;
                let mut buffer = [0; 64 * 1024];
                loop {
                    let size = file.read(&mut buffer)?;
                    if size == 0 {
                        break;
                    }
                    hash.update(&buffer[..size]);
                }
            } else {
                bail!("Unsupported template entry {}", entry.path().display());
            }
        }
        Ok(())
    }

    let mut hash = Sha256::new();
    walk(root, Path::new(""), &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn target_name() -> String {
    let os = match env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}-{arch}")
}

fn target_matches(template: &Template) -> bool {
    template.target == target_name() || template.target == "any"
}

pub fn list(ctx: &Context) -> Result<Value> {
    let mut result = Vec::new();
    for entry in catalog_entries(ctx)? {
        let mut value = serde_json::to_value(entry.template)?;
        value["installed"] = json!(entry.installed.is_some());
        result.push(value);
    }
    Ok(json!(result))
}

pub fn runtimes(ctx: &Context) -> Result<Value> {
    let mut stacks = std::collections::BTreeMap::<&str, Vec<&Template>>::new();
    for template in &ctx.catalog.templates {
        stacks.entry(&template.stack).or_default().push(template);
    }
    std::thread::scope(|scope| {
        let mut checks = Vec::new();
        for (stack, releases) in stacks {
            let sample = releases
                .iter()
                .copied()
                .max_by_key(|release| version(&release.version).unwrap_or_default())
                .context("No starter supports this runtime")?;
            let name = match stack {
                "dotnet" => "C# and .NET",
                "elixir" => "Elixir and Erlang/OTP",
                "go" => "Go",
                "java" => "Java and Maven",
                "python" => "Python",
                "ruby" => "Ruby",
                "rust" => "Rust",
                _ => stack,
            };
            let docs: std::collections::BTreeSet<_> = ctx
                .catalog
                .docs
                .iter()
                .filter(|doc| doc.stack == stack)
                .map(|doc| &doc.id)
                .collect();
            let templates: std::collections::BTreeSet<_> = releases
                .iter()
                .copied()
                .map(|release| &release.id)
                .collect();
            checks.push(scope.spawn(move || {
                let (path, detail) = toolchain(sample);
                json!({
                    "id": stack, "name": name, "toolchain": sample.toolchain,
                    "minimum_version": sample.minimum_version, "available": path.is_some(),
                    "path": path, "detail": detail, "docs": docs, "templates": templates,
                })
            }));
        }
        let rows = checks
            .into_iter()
            .map(|check| {
                check
                    .join()
                    .map_err(|_| anyhow::anyhow!("Runtime inspection failed"))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(json!(rows))
    })
}

fn installation_result(template: &Template, path: &Path) -> Value {
    json!({"id":template.id,"version":template.version,"path":path,"verified":template.verified,"installed":true})
}

fn publish_release(stage: &Stage, template: Template, destination: &Path) -> Result<Value> {
    directory(&stage.0.join("project"))?;
    if fs::symlink_metadata(stage.0.join("installed.json")).is_ok() {
        bail!("Unexpected installed.json in release input");
    }
    let marker = Installed {
        digest: tree_digest(&stage.0)?,
        template,
    };
    fs::write(
        stage.0.join("installed.json"),
        serde_json::to_vec_pretty(&marker)?,
    )?;
    stage.publish(destination)?;
    Ok(installation_result(&marker.template, destination))
}

pub fn install(ctx: &Context, reference: &str) -> Result<Value> {
    let _lock = lock_library(ctx)?;
    let entry = select(ctx, reference, false)?;
    let template = entry.template;
    if let Some(path) = entry.installed {
        verify_release(&path)?;
        return Ok(installation_result(&template, &path));
    }
    let source = source(ctx, &template.path)?;
    let base = installed_base(ctx, &template.id, true)?;
    let destination = base.join(&template.version);
    let stage = Stage::new(&base)?;
    copy_tree(&source, &stage.0, None)?;
    publish_release(&stage, template, &destination)
}

fn candidate_paths(tool: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let override_name = format!("DEVKIT_{}", tool.to_ascii_uppercase().replace('-', "_"));
    if let Some(path) = env::var_os(override_name) {
        return vec![PathBuf::from(path)];
    }
    if let Some(path) = env::var_os("PATH") {
        paths.extend(
            env::split_paths(&path)
                .filter(|p| p.is_absolute())
                .map(|p| p.join(tool)),
        );
    }
    paths.extend(
        ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"]
            .iter()
            .map(|p| Path::new(p).join(tool)),
    );
    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        let runtime = match tool {
            "python3" => "python",
            "mvn" => "maven",
            "mix" => "elixir",
            "erl" => "erlang",
            other => other,
        };
        let installs = home.join(".local/share/mise/installs").join(runtime);
        let mut runtimes = vec![installs.join("latest")];
        if let Ok(dirs) = fs::read_dir(installs) {
            let mut found: Vec<_> = dirs.flatten().map(|entry| entry.path()).collect();
            found.sort();
            runtimes.extend(found.into_iter().rev());
        }
        for runtime in runtimes {
            paths.push(runtime.join("bin").join(tool));
            paths.push(runtime.join(tool));
            if tool == "mvn" {
                if let Ok(dirs) = fs::read_dir(&runtime) {
                    paths.extend(dirs.flatten().map(|e| e.path().join("bin/mvn")));
                }
            }
        }
        if tool == "cargo" {
            let rustup = env::var_os("RUSTUP_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".rustup"));
            if let Ok(dirs) = fs::read_dir(rustup.join("toolchains")) {
                let mut dirs: Vec<_> = dirs.flatten().map(|e| e.path().join("bin/cargo")).collect();
                dirs.sort_by_key(|path| {
                    !path
                        .components()
                        .any(|p| p.as_os_str().to_string_lossy().starts_with("stable-"))
                });
                paths.extend(dirs);
            }
        }
    }
    paths
}

fn known_executable(path: &Path, tool: &str) -> Option<PathBuf> {
    if !path.is_absolute() || (tool == "python3" && path == Path::new("/usr/bin/python3")) {
        return None;
    }
    let actual = path.canonicalize().ok()?;
    let name = actual.file_name()?.to_str()?;
    let metadata = fs::metadata(&actual).ok()?;
    if !metadata.is_file()
        || !executable(&metadata)
        || matches!(name, "rustup" | "mise" | "asdf")
        || path.components().any(|part| part.as_os_str() == "shims")
    {
        return None;
    }
    Some(actual)
}

fn erlang_bin() -> Option<PathBuf> {
    candidate_paths("erl")
        .into_iter()
        .find_map(|path| known_executable(&path, "erl"))
        .and_then(|path| path.parent().map(Path::to_owned))
}

fn jdk_home() -> Option<PathBuf> {
    let output = Command::new("/usr/libexec/java_home")
        .args(["-F", "-v", "26"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
}

fn tool_environment(command: &mut Command, tool: &str, path: &Path) -> Result<()> {
    let mut directories = vec![path
        .parent()
        .context("Invalid toolchain path")?
        .to_path_buf()];
    if tool == "elixir" {
        directories.extend(erlang_bin());
    }
    if matches!(tool, "mvn" | "dotnet" | "elixir") {
        let python = candidate_paths("python3")
            .into_iter()
            .find_map(|path| known_executable(&path, "python3"));
        if let Some(parent) = python.as_ref().and_then(|path| path.parent()) {
            directories.push(parent.to_path_buf());
        }
    }
    let original = env::var_os("PATH").unwrap_or_default();
    directories.extend(env::split_paths(&original).filter(|path| path.is_absolute()));
    directories.extend(
        [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ]
        .into_iter()
        .map(PathBuf::from),
    );
    command.env("PATH", env::join_paths(directories)?);
    if tool == "mvn" {
        if let Some(java) = jdk_home() {
            command.env("JAVA_HOME", java);
        }
    }
    Ok(())
}

fn probe(tool: &str, path: &Path) -> Result<String> {
    let args: &[&str] = match tool {
        "go" => &["version"],
        "python3" => &["-I", "-S", "-c", "import platform,sys; print(platform.python_version(), sys.implementation.name, platform.machine())"],
        _ => &["--version"],
    };
    let mut command = Command::new(path);
    tool_environment(&mut command, tool, path)?;
    let mut child = command
        .args(args)
        .current_dir("/")
        .env("GOTOOLCHAIN", "local")
        .env("GOWORK", "off")
        .env("GOPROXY", "off")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Version check timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output()?;
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        bail!("Version check failed")
    }
    Ok(text.trim().lines().take(6).collect::<Vec<_>>().join(" "))
}

fn runtime_numbers(value: &str) -> Option<Vec<u64>> {
    let mut value = value.trim_start_matches(|c: char| !c.is_ascii_digit());
    if value.is_empty() {
        return None;
    }
    value = value.trim_end_matches('.');
    let end = value
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(value.len());
    let numbers: Vec<_> = value[..end]
        .split('.')
        .filter_map(|part| part.parse::<u64>().ok())
        .collect();
    (!numbers.is_empty()).then_some(numbers)
}

fn meets_minimum(template: &Template, observed: &str) -> bool {
    if template.minimum_version.is_empty() {
        return true;
    }
    let line = if template.toolchain == "elixir" {
        observed.split("Elixir ").nth(1).unwrap_or(observed)
    } else {
        observed
    };
    let Some(mut actual) = runtime_numbers(line) else {
        return false;
    };
    let Some(mut required) = runtime_numbers(&template.minimum_version) else {
        return false;
    };
    let width = actual.len().max(required.len()).max(3);
    actual.resize(width, 0);
    required.resize(width, 0);
    if actual < required {
        return false;
    }
    match template.toolchain.as_str() {
        "python3" => actual[..2] == required[..2] && observed.contains("cpython arm64"),
        "dotnet" => actual[..2] == required[..2] && actual[2] / 100 == required[2] / 100,
        "mvn" => {
            let java = observed
                .split("Java version: ")
                .nth(1)
                .and_then(runtime_numbers);
            actual[0] == 3 && java.is_some_and(|v| v[0] == 26)
        }
        "elixir" => actual[..2] == required[..2] && observed.starts_with("Erlang/OTP 29 "),
        _ => true,
    }
}

fn toolchain(template: &Template) -> (Option<PathBuf>, String) {
    let tool = template.toolchain.as_str();
    if tool.is_empty() || tool.contains('/') || tool.contains('\\') {
        return (None, "Invalid executable name".to_owned());
    }
    let mut incompatible = None;
    let mut visited = std::collections::BTreeSet::new();
    for candidate in candidate_paths(tool) {
        let Some(path) = known_executable(&candidate, tool) else {
            continue;
        };
        if !visited.insert(path.clone()) {
            continue;
        }
        let Ok(actual) = probe(tool, &path) else {
            continue;
        };
        let detail = format!("{} · {actual}", path.display());
        if meets_minimum(template, &actual) {
            return (Some(path), detail);
        }
        incompatible.get_or_insert(detail);
    }
    let companion = match tool {
        "python3" => " (arm64 CPython, matching major/minor)",
        "mvn" => " with JDK 26",
        "elixir" => " with Erlang/OTP 29",
        "dotnet" => " in the same SDK feature band",
        _ => "",
    };
    (
        None,
        match incompatible {
            Some(found) => format!(
                "Requires {tool} {}{companion}. Found {found}",
                template.minimum_version
            ),
            None => format!(
                "Install {tool} {}{companion} or set DEVKIT_{} to its executable path.",
                template.minimum_version,
                tool.to_ascii_uppercase()
            ),
        },
    )
}

fn check(name: &str, ok: bool, detail: impl Into<String>) -> Value {
    json!({"name":name,"ok":ok,"detail":detail.into()})
}

pub fn doctor(ctx: &Context, reference: &str) -> Result<Value> {
    let entry = select(ctx, reference, true)?;
    let t = &entry.template;
    let (tool, detail) = toolchain(t);
    let mut checks = vec![
        check(
            "Installed release",
            entry.installed.is_some(),
            entry
                .installed
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| format!("Install {}@{}", t.id, t.version)),
        ),
        check(
            "Target",
            target_matches(t),
            format!(
                "Bundle targets {}. This app is running on {}.",
                t.target,
                target_name()
            ),
        ),
        check("Toolchain", tool.is_some(), detail),
        check(
            "Offline verification",
            t.verified,
            if t.verified {
                "Offline test passed with the packaged dependencies and isolated caches."
            } else {
                "Offline verification has not been recorded for this release."
            },
        ),
    ];
    if let Some(path) = &entry.installed {
        let intact = verify_release(path).is_ok();
        checks.push(check(
            "Bundle integrity",
            intact,
            if intact {
                "Files match the installed release digest."
            } else {
                "Files differ from the installed release digest."
            },
        ));
    }
    Ok(json!({"ready":checks.iter().all(|c|c["ok"] == true),"checks":checks,"template":t}))
}

fn shell_word(path: &Path) -> Result<String> {
    let path = path.to_str().context("Toolchain path must be UTF-8")?;
    Ok(format!("'{}'", path.replace('\'', "'\\''")))
}

fn project_test_wrapper(project: &Path, template: &Template, tool: &Path) -> Result<()> {
    use std::fmt::Write as _;

    let mut script = String::from("#!/bin/sh\n# Run the bundled test script with the toolchain selected by Devline.\nset -eu\ncd \"$(dirname \"$0\")\"\n");
    let tool_path = shell_word(tool)?;
    writeln!(
        script,
        "if [ -z \"${{DEVKIT_TOOLCHAIN:-}}\" ] && [ -x {tool_path} ]; then"
    )?;
    writeln!(script, "    export DEVKIT_TOOLCHAIN={tool_path}")?;
    writeln!(script, "fi")?;
    if template.toolchain == "elixir" {
        if let Some(erl) = erlang_bin() {
            let dir = shell_word(&erl)?;
            writeln!(
                script,
                "if [ -d {dir} ]; then export PATH={dir}${{PATH:+:$PATH}}; fi"
            )?;
        }
    }
    if template.toolchain == "mvn" {
        if let Some(java) = jdk_home() {
            let bin = shell_word(&java.join("bin/java"))?;
            let home = shell_word(&java)?;
            writeln!(script, "if [ -n \"${{DEVKIT_JAVA_HOME:-}}\" ]; then")?;
            writeln!(script, "    export JAVA_HOME=\"$DEVKIT_JAVA_HOME\"")?;
            writeln!(script, "elif [ -x {bin} ]; then")?;
            writeln!(script, "    export JAVA_HOME={home}")?;
            writeln!(script, "fi")?;
        }
    }
    script.push_str("if [ -n \"${DEVKIT_TOOLCHAIN:-}\" ] && [ -x \"$DEVKIT_TOOLCHAIN\" ]; then\n");
    script.push_str("    export PATH=\"$(dirname \"$DEVKIT_TOOLCHAIN\")${PATH:+:$PATH}\"\nfi\n");
    script.push_str("exec /bin/sh ./test-offline.sh\n");
    let wrapper = project.join("devkit-test.sh");
    use std::io::Write as _;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&wrapper)?;
    file.write_all(script.as_bytes())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(wrapper, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

pub fn create(ctx: &Context, reference: &str, destination: &Path, dry_run: bool) -> Result<Value> {
    let _lock = lock_library(ctx)?;
    let entry = select(ctx, reference, true)?;
    let template = entry.template;
    let input = entry.installed.with_context(|| {
        format!(
            "Install {}@{} before creating a project",
            template.id, template.version
        )
    })?;
    if !target_matches(&template) {
        bail!(
            "Template targets {}. This app is running on {}.",
            template.target,
            target_name()
        );
    }
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .context("Choose a project directory with a valid name")?;
    if !slug(name) {
        bail!("Use a project name starting with a lowercase letter and containing lowercase letters, digits, or hyphens");
    }
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .context("The destination's parent folder must exist")?;
    let destination = parent.join(name);
    match fs::symlink_metadata(&destination) {
        Ok(_) => bail!("The destination already exists: {}", destination.display()),
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e.into()),
        Err(_) => (),
    }
    if parent.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a project directory outside the Devline library");
    }
    if !template.verified {
        bail!(
            "Template {}@{} has not passed offline verification",
            template.id,
            template.version
        );
    }
    let (tool, detail) = toolchain(&template);
    let tool = tool.context(detail)?;
    let marker = verify_release(&input)?;
    let has_test_script = template.test_command == "./test-offline.sh"
        && regular(&input.join("project/test-offline.sh")).is_ok();
    let test_command = if has_test_script {
        "./devkit-test.sh"
    } else {
        &template.test_command
    };
    if !dry_run {
        let stage = Stage::new(&parent)?;
        copy_tree(&input.join("project"), &stage.0, Some(name))?;
        if has_test_script {
            project_test_wrapper(&stage.0, &template, &tool)?;
        }
        fs::write(
            stage.0.join(".devkit-origin.json"),
            serde_json::to_vec_pretty(&json!({
                "template": template.id,
                "version": template.version,
                "target": template.target,
                "digest": marker.digest,
            }))?,
        )?;
        stage.publish(&destination)?;
    }
    Ok(
        json!({"path":destination,"template":template.id,"version":template.version,"test_command":test_command,"toolchain_path":tool,"dry_run":dry_run}),
    )
}

fn verify_release(path: &Path) -> Result<Installed> {
    let marker = read_marker(path)?;
    if tree_digest(path)? != marker.digest {
        bail!(
            "Installed template {}@{} is damaged",
            marker.template.id,
            marker.template.version
        );
    }
    Ok(marker)
}

fn completed(command: &mut Command, stage: &str) -> Result<()> {
    let result = command
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("Could not start {stage}"))?;
    if result.status.success() {
        return Ok(());
    }
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let last: Vec<_> = diagnostic.lines().rev().take(12).collect();
    bail!(
        "{stage} failed. {}",
        last.into_iter().rev().collect::<Vec<_>>().join("\n")
    );
}

fn verify_build(project: &Path, work: &Path, tool: &Path, tool_name: &str) -> Result<()> {
    let copy = work.join("offline-test");
    fs::create_dir(&copy)?;
    copy_tree(project, &copy, Some("devkit-test"))?;
    let script = copy.join("test-offline.sh");
    regular(&script).context("An offline verification script is required")?;
    let home = work.join("offline-home");
    fs::create_dir(&home)?;
    const POLICY: &str = "(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip \"localhost:*\"))";
    let mut verifier = Command::new("/usr/bin/sandbox-exec");
    verifier.env_clear();
    tool_environment(&mut verifier, tool_name, tool)?;
    completed(
        verifier
            .args(["-p", POLICY, "/bin/sh"])
            .arg(script)
            .current_dir(&copy)
            .env("HOME", &home)
            .env("TMPDIR", work)
            .env("DEVKIT_TOOLCHAIN", tool)
            .env("MIX_HOME", home.join("mix"))
            .env("HEX_HOME", home.join("hex"))
            .env("GOCACHE", home.join("go-build"))
            .env("GOMODCACHE", home.join("go-mod"))
            .env("CARGO_HOME", home.join("cargo"))
            .env("CARGO_TARGET_DIR", home.join("target"))
            .env("DOTNET_CLI_HOME", &home)
            .env("NUGET_PACKAGES", home.join("nuget"))
            .env("NUGET_HTTP_CACHE_PATH", home.join("nuget-http")),
        "network-denied verification",
    )
}

fn next_release_version(ctx: &Context, template: &Template) -> Result<String> {
    let [major, minor, base_patch] = version(&template.version)?;
    let mut latest_patch = base_patch;
    for entry in catalog_entries(ctx)? {
        if entry.template.id != template.id {
            continue;
        }
        let [entry_major, entry_minor, patch] = version(&entry.template.version)?;
        if entry_major == major && entry_minor == minor {
            latest_patch = latest_patch.max(patch);
        }
    }
    let patch = latest_patch
        .checked_add(1)
        .context("Template version overflow")?;
    Ok(format!("{major}.{minor}.{patch}"))
}

pub fn build_latest(ctx: &Context, reference: &str) -> Result<Value> {
    let entry = select(ctx, reference, false)?;
    let mut template = entry.template;
    let source = source(ctx, &template.path)?;
    let recipe = source.join("build-latest.sh");
    regular(&recipe).context("This template does not provide a latest-version recipe")?;
    if !target_matches(&template) {
        bail!("The template recipe targets {}", template.target)
    }
    let (tool, detail) = toolchain(&template);
    let tool = tool.context(detail)?;
    let base = installed_base(ctx, &template.id, true)?;
    let stage = Stage::new(&base)?;
    copy_tree(&source, &stage.0, None)?;
    let project = stage.0.join("project");
    let mut builder = Command::new("/bin/sh");
    tool_environment(&mut builder, &template.toolchain, &tool)?;
    completed(
        builder
            .arg(&recipe)
            .arg(&project)
            .current_dir(&stage.0)
            .env("DEVKIT_TOOLCHAIN", &tool),
        "template dependency refresh",
    )?;
    let verification = Stage::new(&base)?;
    verify_build(&project, &verification.0, &tool, &template.toolchain)?;
    let _lock = lock_library(ctx)?;
    template.version = next_release_version(ctx, &template)?;
    let output = base.join(&template.version);
    template.verified = true;
    template.description = format!(
        "Locally rebuilt {} with resolved and offline-tested dependencies.",
        template.name
    );
    publish_release(&stage, template, &output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    struct Fixture {
        _root: Stage,
        ctx: Context,
    }

    impl Fixture {
        fn new() -> Result<Self> {
            let root = Stage::new(&env::temp_dir().canonicalize()?)?;
            let home = root.0.join("library");
            let catalog_dir = root.0.join("catalog");
            fs::create_dir(&home)?;
            let mut templates = Vec::new();
            for version in ["1.0.0", "1.1.0"] {
                let path = format!("templates/fixture/starter/{version}");
                let project = catalog_dir.join(&path).join("project");
                fs::create_dir_all(&project)?;
                fs::write(
                    project.join("main.txt"),
                    format!("{version} __PROJECT_NAME__"),
                )?;
                templates.push(Template {
                    id: "fixture/starter".into(),
                    name: "Starter fixture".into(),
                    stack: "fixture".into(),
                    version: version.into(),
                    description: "Fixture".into(),
                    app_type: "cli".into(),
                    path,
                    toolchain: "true".into(),
                    minimum_version: String::new(),
                    test_command: "./test-offline.sh".into(),
                    docs: Vec::new(),
                    verified: true,
                    target: target_name(),
                });
            }
            let ctx = Context {
                home,
                catalog_dir,
                catalog: Catalog {
                    schema_version: 1,
                    docs: Vec::new(),
                    templates,
                },
            };
            Ok(Self { _root: root, ctx })
        }
    }

    #[test]
    fn exact_and_default_versions_generate_without_overwriting() -> Result<()> {
        let fixture = Fixture::new()?;
        let first = "fixture/starter@1.0.0";
        let latest = "fixture/starter";
        install(&fixture.ctx, first)?;
        install(&fixture.ctx, latest)?;
        let older = fixture._root.0.join("older-starter");
        let current = fixture._root.0.join("current-starter");
        let dry_run = create(&fixture.ctx, first, &older, true)?;
        assert_eq!(dry_run["version"], "1.0.0");
        assert!(!older.exists());
        create(&fixture.ctx, first, &older, false)?;
        create(&fixture.ctx, latest, &current, false)?;
        assert_eq!(
            fs::read_to_string(older.join("main.txt"))?,
            "1.0.0 older-starter"
        );
        assert_eq!(
            fs::read_to_string(current.join("main.txt"))?,
            "1.1.0 current-starter"
        );
        let error = create(&fixture.ctx, latest, &older, false).unwrap_err();
        assert!(error.to_string().contains("already exists"));
        assert_eq!(
            fs::read_to_string(older.join("main.txt"))?,
            "1.0.0 older-starter"
        );
        Ok(())
    }

    #[test]
    fn altered_payload_is_rejected_before_writing_project() -> Result<()> {
        let fixture = Fixture::new()?;
        let id = "fixture/starter";
        install(&fixture.ctx, id)?;
        let installed = fixture
            .ctx
            .home
            .join("templates/fixture/starter/1.1.0/project/main.txt");
        fs::write(installed, "1.1.1 __PROJECT_NAME__")?;
        let destination = fixture._root.0.join("damaged-starter");
        for dry_run in [true, false] {
            let error = create(&fixture.ctx, id, &destination, dry_run).unwrap_err();
            assert!(error.to_string().contains("damaged"));
            assert!(!destination.exists());
        }
        assert_eq!(doctor(&fixture.ctx, id)?["ready"], false);
        assert!(install(&fixture.ctx, id)
            .unwrap_err()
            .to_string()
            .contains("damaged"));
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn a_symlink_never_installs_a_partial_release() -> Result<()> {
        let fixture = Fixture::new()?;
        let id = "fixture/starter";
        let catalog_project = fixture
            .ctx
            .catalog_dir
            .join("templates/fixture/starter/1.1.0/project");
        std::os::unix::fs::symlink(
            catalog_project.join("main.txt"),
            catalog_project.join("linked.txt"),
        )?;
        let error = install(&fixture.ctx, id).unwrap_err();
        assert!(error
            .to_string()
            .contains("not a regular file or directory"));
        let install_base = fixture.ctx.home.join("templates/fixture/starter");
        assert!(fs::read_dir(install_base)?.next().is_none());
        Ok(())
    }

    #[test]
    fn unsuccessful_build_keeps_prior_release_and_cleans_stage() -> Result<()> {
        let fixture = Fixture::new()?;
        let id = "fixture/starter";
        install(&fixture.ctx, id)?;
        let recipe = fixture
            .ctx
            .catalog_dir
            .join("templates/fixture/starter/1.1.0/build-latest.sh");
        fs::write(
            recipe,
            "printf partial > \"$1/main.txt\"\nprintf 'dependency refresh failed\\n' >&2\nexit 4\n",
        )?;
        let error = build_latest(&fixture.ctx, id).unwrap_err();
        assert!(error.to_string().contains("dependency refresh failed"));
        let base = fixture.ctx.home.join("templates/fixture/starter");
        let releases = fs::read_dir(base)?
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(releases, ["1.1.0"]);
        let target = fixture._root.0.join("still-ready");
        create(&fixture.ctx, id, &target, false)?;
        assert_eq!(
            fs::read_to_string(target.join("main.txt"))?,
            "1.1.0 still-ready"
        );
        Ok(())
    }

    #[test]
    fn latest_refresh_preserves_the_selected_release_line() -> Result<()> {
        let fixture = Fixture::new()?;
        let entry = select(&fixture.ctx, "fixture/starter@1.0.0", false)?;
        assert_eq!(
            next_release_version(&fixture.ctx, &entry.template)?,
            "1.0.1"
        );
        let head = select(&fixture.ctx, "fixture/starter", false)?;
        assert_eq!(next_release_version(&fixture.ctx, &head.template)?, "1.1.1");

        let base = installed_base(&fixture.ctx, &entry.template.id, true)?;
        let stage = Stage::new(&base)?;
        let input = source(&fixture.ctx, &entry.template.path)?;
        copy_tree(&input, &stage.0, None)?;
        let mut local = entry.template.clone();
        local.version = "1.0.5".into();
        publish_release(&stage, local, &base.join("1.0.5"))?;
        assert_eq!(
            next_release_version(&fixture.ctx, &entry.template)?,
            "1.0.6"
        );
        Ok(())
    }

    #[test]
    fn generated_test_wrapper_quotes_and_preserves_tool_overrides() -> Result<()> {
        let fixture = Fixture::new()?;
        let project = &fixture._root.0;
        let unusual_tool = project.join("tool's directory").join("test-tool");
        fs::create_dir(unusual_tool.parent().unwrap())?;
        fs::copy("/usr/bin/true", &unusual_tool)?;
        fs::write(
            project.join("test-offline.sh"),
            "printf '%s' \"$DEVKIT_TOOLCHAIN\"\n",
        )?;
        project_test_wrapper(project, &fixture.ctx.catalog.templates[0], &unusual_tool)?;
        for (override_value, expected) in [
            (None, unusual_tool.as_path()),
            (Some("/usr/bin/true"), Path::new("/usr/bin/true")),
        ] {
            let mut command = Command::new("/bin/sh");
            command
                .arg(project.join("devkit-test.sh"))
                .env_clear()
                .env("PATH", "/usr/bin:/bin");
            if let Some(path) = override_value {
                command.env("DEVKIT_TOOLCHAIN", path);
            }
            let output = command.output()?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8(output.stdout)?,
                expected.to_str().unwrap()
            );
        }
        Ok(())
    }
}
