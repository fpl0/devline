use crate::{lock_library, tasks, text_arg, Context, Template};
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
pub(crate) struct Installed {
    pub(crate) template: Template,
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) verified_at: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct Verification {
    at: u64,
    digest: String,
    toolchain: String,
    target: String,
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

pub(crate) fn id_parts(id: &str) -> Result<(&str, &str)> {
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

pub(crate) fn library(ctx: &Context, path: &Path, create: bool) -> Result<PathBuf> {
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

pub(crate) fn installed_base(ctx: &Context, id: &str, create: bool) -> Result<PathBuf> {
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
    let base = library(ctx, Path::new("templates"), false)?;
    if let Ok(stacks) = fs::read_dir(base) {
        for stack in stacks {
            let stack = stack?;
            if !stack.file_type()?.is_dir() {
                continue;
            }
            let label = stack.file_name();
            let Some(stack_name) = label.to_str().filter(|name| slug(name)) else {
                continue;
            };
            for family in fs::read_dir(stack.path())? {
                let family = family?;
                if !family.file_type()?.is_dir() {
                    continue;
                }
                let label = family.file_name();
                if let Some(name) = label.to_str().filter(|name| slug(name)) {
                    ids.insert(format!("{stack_name}/{name}"));
                }
            }
        }
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

pub(crate) struct Stage(pub(crate) PathBuf);

impl Stage {
    pub(crate) fn new(parent: &Path) -> Result<Self> {
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

    pub(crate) fn publish(&self, destination: &Path) -> Result<()> {
        rename_exclusive(&self.0, destination)
            .with_context(|| format!("Could not create {}", destination.display()))
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        if fs::remove_dir_all(&self.0)
            .is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied)
        {
            #[cfg(unix)]
            {
                fn make_removable(path: &Path) {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(info) = fs::symlink_metadata(path) {
                        if info.is_dir() {
                            let _ = fs::set_permissions(
                                path,
                                fs::Permissions::from_mode(info.permissions().mode() | 0o700),
                            );
                            if let Ok(children) = fs::read_dir(path) {
                                for child in children.flatten() {
                                    make_removable(&child.path());
                                }
                            }
                        }
                    }
                }
                make_removable(&self.0);
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn rename_exclusive(from: &Path, to: &Path) -> io::Result<()> {
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
pub(crate) fn rename_exclusive(from: &Path, to: &Path) -> io::Result<()> {
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

pub(crate) fn copy_tree(from: &Path, to: &Path, name: Option<&str>) -> Result<()> {
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

pub(crate) fn tree_digest(root: &Path) -> Result<String> {
    fn walk(root: &Path, relative: &Path, hash: &mut Sha256, buffer: &mut [u8]) -> Result<()> {
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
                walk(root, &path, hash, buffer)?;
            } else if metadata.is_file() {
                hash.update([1, u8::from(executable(&metadata))]);
                hash.update(metadata.len().to_le_bytes());
                let mut file = fs::File::open(entry.path())?;
                loop {
                    let size = file.read(buffer)?;
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
    // Swift's cooperative worker threads have small stacks. Reuse one heap buffer
    // instead of reserving 64 KiB for every nested dependency directory.
    let mut buffer = vec![0; 64 * 1024];
    walk(root, Path::new(""), &mut hash, &mut buffer)?;
    Ok(format!("{:x}", hash.finalize()))
}

pub(crate) fn target_name() -> String {
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

pub(crate) fn target_matches(template: &Template) -> bool {
    (template.target == target_name() || template.target == "any")
        && template.minimum_macos.as_deref().is_none_or(|minimum| {
            macos_version()
                .and_then(os_version)
                .zip(os_version(minimum))
                .is_some_and(|(actual, minimum)| actual >= minimum)
        })
}

fn os_version(text: &str) -> Option<[u32; 3]> {
    let mut parsed = [0; 3];
    let parts: Vec<_> = text.split('.').collect();
    if parts.len() > parsed.len() {
        return None;
    }
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        parsed[index] = part.parse().ok()?;
    }
    Some(parsed)
}

fn macos_version() -> Option<&'static str> {
    static VERSION: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    VERSION
        .get_or_init(|| {
            if !cfg!(target_os = "macos") {
                return None;
            }
            let output = Command::new("/usr/bin/sw_vers")
                .arg("-productVersion")
                .env("SYSTEM_VERSION_COMPAT", "0")
                .output()
                .ok()?;
            let actual = String::from_utf8(output.stdout).ok()?.trim().to_owned();
            (output.status.success() && os_version(&actual).is_some()).then_some(actual)
        })
        .as_deref()
}

pub(crate) fn target_detail(template: &Template) -> String {
    let target = target_name();
    if let Some(minimum) = &template.minimum_macos {
        format!(
            "Requires {} and macOS {minimum} or newer. Found {target}, macOS {}.",
            template.target,
            macos_version().unwrap_or("version unavailable")
        )
    } else {
        format!(
            "Bundle targets {}. This app is running on {target}.",
            template.target
        )
    }
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
    runtime_reports(ctx, &[])
}

pub(crate) fn runtime_reports(ctx: &Context, selected: &[String]) -> Result<Value> {
    let mut stacks = std::collections::BTreeMap::<&str, Vec<&Template>>::new();
    for template in &ctx.catalog.templates {
        if !selected.is_empty() && !selected.contains(&template.stack) {
            continue;
        }
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
                "swift" => "Swift and SwiftUI",
                "node" => "JavaScript and TypeScript / Node.js",
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
                let report = toolchain_report(sample);
                json!({
                    "id": stack, "name": name, "toolchain": sample.toolchain,
                    "minimum_version": sample.minimum_version, "available": report["available"],
                    "path": report["path"], "detail": report["detail"], "version":report["version"],
                    "docs": docs, "templates": templates,
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

fn release_size(path: &Path) -> Result<(u64, u64)> {
    let mut result = (0, 0);
    for child in fs::read_dir(path)? {
        let child = child?;
        let metadata = child.metadata()?;
        if child.file_type()?.is_dir() {
            let (files, bytes) = release_size(&child.path())?;
            result.0 += files;
            result.1 += bytes;
        } else if child.file_type()?.is_file() && child.file_name() != "installed.json" {
            result.0 += 1;
            result.1 += metadata.len();
        }
    }
    Ok(result)
}

fn dependency_inventory(project: &Path, stack: &str) -> Result<Vec<Value>> {
    let mut dependencies = std::collections::BTreeSet::new();
    let mut add = |name: &str, version: &str| {
        if !name.is_empty() && !version.is_empty() {
            dependencies.insert((name.to_owned(), version.to_owned()));
        }
    };
    let read = |file: &str| fs::read_to_string(project.join(file)).unwrap_or_default();
    match stack {
        "node" => {
            let value =
                serde_json::from_str::<Value>(&read("package-lock.json")).unwrap_or_default();
            if let Some(packages) = value["packages"].as_object() {
                for (path, package) in packages {
                    if let Some(name) = path.rsplit_once("node_modules/").map(|(_, name)| name) {
                        add(name, package["version"].as_str().unwrap_or_default());
                    }
                }
            }
        }
        "ruby" => {
            for line in read("Gemfile.lock")
                .lines()
                .filter(|line| line.starts_with("    ") && !line.starts_with("     "))
            {
                if let Some((name, rest)) = line.trim().split_once(" (") {
                    if let Some(version) = rest.strip_suffix(')') {
                        add(name, version);
                    }
                }
            }
        }
        "go" => {
            for line in read("vendor/modules.txt")
                .lines()
                .filter_map(|line| line.strip_prefix("# "))
            {
                let mut parts = line.split_whitespace();
                if let (Some(name), Some(version)) = (parts.next(), parts.next()) {
                    if version.starts_with('v') {
                        add(name, version);
                    }
                }
            }
        }
        "python" => {
            for line in read("requirements.txt").lines() {
                if let Some((name, version)) = line.split_once("==") {
                    add(name, version.split_whitespace().next().unwrap_or_default());
                }
            }
        }
        "rust" => {
            for package in read("Cargo.lock")
                .split("[[package]]")
                .filter(|entry| entry.contains("source = "))
            {
                let field = |key: &str| {
                    package
                        .lines()
                        .find_map(|line| line.strip_prefix(key)?.strip_suffix('"'))
                        .unwrap_or_default()
                };
                add(field("name = \""), field("version = \""));
            }
        }
        "dotnet" => {
            let value =
                serde_json::from_str::<Value>(&read("packages.lock.json")).unwrap_or_default();
            if let Some(frameworks) = value["dependencies"].as_object() {
                for dependencies in frameworks.values().filter_map(Value::as_object) {
                    for (name, metadata) in dependencies {
                        add(name, metadata["resolved"].as_str().unwrap_or_default());
                    }
                }
            }
        }
        "elixir" => {
            for line in read("vendor/upstream-lock.exs")
                .lines()
                .filter(|line| line.contains("{:hex,"))
            {
                let mut parts = line.split('"');
                if let (Some(name), Some(version)) = (parts.nth(1), parts.nth(1)) {
                    add(name, version);
                }
            }
        }
        "java" => {
            fn jars(
                root: &Path,
                path: &Path,
                found: &mut std::collections::BTreeSet<(String, String)>,
            ) -> Result<()> {
                for child in fs::read_dir(path)? {
                    let child = child?;
                    let path = child.path();
                    if child.file_type()?.is_dir() {
                        jars(root, &path, found)?;
                    } else if path.extension().is_some_and(|ext| ext == "jar") {
                        let components: Vec<_> = path
                            .strip_prefix(root)?
                            .iter()
                            .filter_map(|p| p.to_str())
                            .collect();
                        if components.len() >= 4 {
                            let version = components[components.len() - 2];
                            let artifact = components[components.len() - 3];
                            let group = components[..components.len() - 3].join(".");
                            found.insert((format!("{group}:{artifact}"), version.to_owned()));
                        }
                    }
                }
                Ok(())
            }
            let root = project.join("maven-repository");
            if root.is_dir() {
                jars(&root, &root, &mut dependencies)?;
            }
        }
        _ => {}
    }
    Ok(dependencies
        .into_iter()
        .map(|(name, version)| json!({"name":name,"version":version}))
        .collect())
}

pub fn inspect(ctx: &Context, reference: &str) -> Result<Value> {
    let (marker, path) = bundle_source(ctx, reference)?;
    let installed = path.starts_with(ctx.home.canonicalize()?);
    let (file_count, size_bytes) = release_size(&path)?;
    let project = path.join("project");
    let mut contents = fs::read_dir(&project)?.map(|entry| {
        let entry = entry?;
        Ok(json!({"name":entry.file_name().to_string_lossy(),"kind":if entry.file_type()?.is_dir() { "directory" } else { "file" }}))
    }).collect::<Result<Vec<_>>>()?;
    contents.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let dependencies = dependency_inventory(&project, &marker.template.stack)?;
    let (toolchain, detail) = toolchain(&marker.template);
    let evidence = load_verification(ctx, &marker.template, &marker.digest, &detail)?;
    Ok(json!({
        "template":marker.template, "installed":installed, "digest":marker.digest,
        "size_bytes":size_bytes, "file_count":file_count, "contents":contents,
        "dependencies":dependencies, "verified_at":marker.verified_at,
        "verification":evidence, "ready_offline":installed && toolchain.is_some() && evidence.is_some(),
    }))
}

pub(crate) fn publish_release(
    stage: &Stage,
    template: Template,
    destination: &Path,
    verified_at: Option<u64>,
) -> Result<Value> {
    directory(&stage.0.join("project"))?;
    if fs::symlink_metadata(stage.0.join("installed.json")).is_ok() {
        bail!("Unexpected installed.json in release input");
    }
    let marker = Installed {
        digest: tree_digest(&stage.0)?,
        template,
        verified_at,
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
    publish_release(&stage, template, &destination, None)
}

fn candidate_paths(tool: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let override_name = format!("DEVKIT_{}", tool.to_ascii_uppercase().replace('-', "_"));
    if let Some(path) = env::var_os(override_name) {
        return vec![PathBuf::from(path)];
    }
    if tool == "swift" {
        paths.extend(xcode_path(&["--find", "swift"]));
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
    if !path.is_absolute()
        || (tool == "python3" && path == Path::new("/usr/bin/python3"))
        || (tool == "swift" && path == Path::new("/usr/bin/swift"))
    {
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
    // Swift dispatches its driver mode using argv[0]. Resolving its symlink to
    // swift-frontend would make `swift package` and `swift build` stop working.
    Some(if tool == "swift" {
        path.to_owned()
    } else {
        actual
    })
}

fn xcode_path(args: &[&str]) -> Option<PathBuf> {
    let output = Command::new("/usr/bin/xcrun")
        .arg("--no-cache")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    (output.status.success() && path.is_absolute() && path.exists()).then_some(path)
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

pub(crate) fn tool_environment(command: &mut Command, tool: &str, path: &Path) -> Result<()> {
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
    if matches!(tool, "swift" | "cargo") {
        for key in ["DEVELOPER_DIR", "TOOLCHAINS"] {
            if let Some(value) = env::var_os(key) {
                command.env(key, value);
            }
        }
        let sdk = xcode_path(&["--sdk", "macosx", "--show-sdk-path"]);
        if tool == "swift" && sdk.is_none() {
            bail!("SwiftUI requires a local macOS SDK. Select Xcode or Command Line Tools with xcode-select; Devline does not install toolchains.");
        }
        if let Some(sdk) = sdk {
            command.env("SDKROOT", sdk);
        }
    }
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
    let mut detail = text.trim().lines().take(6).collect::<Vec<_>>().join(" ");
    if matches!(tool, "swift" | "cargo") {
        if let Some(sdk) = xcode_path(&["--sdk", "macosx", "--show-sdk-path"]) {
            detail.push_str(&format!("; SDK {}", sdk.display()));
        }
    }
    Ok(detail)
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
    let line = match template.toolchain.as_str() {
        "elixir" => observed.split("Elixir ").nth(1).unwrap_or(observed),
        "swift" => observed.split("Swift version ").nth(1).unwrap_or(observed),
        _ => observed,
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
    if template.id == "rust/tauri-desktop"
        && xcode_path(&["--sdk", "macosx", "--show-sdk-path"]).is_none()
    {
        return (None, "The Tauri desktop starter needs Apple’s macOS SDK and native build tools. Select an installed Xcode or Command Line Tools with xcode-select. Devline only detects these tools.".into());
    }
    if template.id == "swift/swiftui-app"
        && !xcode_path(&["--find", "xcodebuild"])
            .and_then(|path| path.ancestors().nth(3).map(Path::to_owned))
            .is_some_and(|developer| developer.join("Platforms/MacOSX.platform/Developer/Library/Frameworks/Testing.framework/Modules/Testing.swiftmodule").is_dir())
    {
        return (None, "This SwiftUI starter needs Xcode 16 or newer, including Swift Testing and a macOS SDK. Select an installed Xcode in Settings → Locations → Command Line Tools, or set DEVELOPER_DIR. Devline only detects these tools.".into());
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
        "swift" => " with Xcode and a macOS SDK selected by xcode-select",
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
        check("Target", target_matches(t), target_detail(t)),
        check("Toolchain", tool.is_some(), &detail),
    ];
    let mut evidence = None;
    if let Some(path) = &entry.installed {
        let marker = verify_release(path);
        let intact = marker.is_ok();
        checks.push(check(
            "Bundle integrity",
            intact,
            if intact {
                "Files match the installed release digest."
            } else {
                "Files differ from the installed release digest."
            },
        ));
        if let Ok(marker) = marker {
            evidence = load_verification(ctx, t, &marker.digest, &detail)?;
        }
    }
    checks.push(check(
        "Offline verification",
        t.verified || evidence.is_some(),
        if evidence.is_some() {
            "The installed release passed network-denied tests with this toolchain."
        } else if t.verified {
            "Release qualification passed with isolated caches. Check this Mac to confirm readiness."
        } else {
            "Offline verification has not been recorded for this release."
        },
    ));
    let ready = checks.iter().all(|c| c["ok"] == true);
    let ready_offline = ready && evidence.is_some();
    Ok(json!({
        "ready": ready, "ready_offline": ready_offline,
        "status": if ready_offline { "ready_offline" } else if ready { "verification_required" } else { "blocked" },
        "checks":checks, "template":t, "verification":evidence
    }))
}

fn readiness_file(ctx: &Context, template: &Template, create: bool) -> Result<PathBuf> {
    let (stack, name) = id_parts(&template.id)?;
    version(&template.version)?;
    let path = library(ctx, &Path::new("readiness").join(stack).join(name), create)?;
    Ok(path.join(format!("{}.json", template.version)))
}

fn load_verification(
    ctx: &Context,
    template: &Template,
    digest: &str,
    toolchain: &str,
) -> Result<Option<Verification>> {
    let path = readiness_file(ctx, template, false)?;
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        _ => return Ok(None),
    }
    let evidence = serde_json::from_slice::<Verification>(&fs::read(path)?).ok();
    Ok(evidence.filter(|entry| {
        entry.digest == digest && entry.toolchain == toolchain && entry.target == target_name()
    }))
}

fn record_verification(
    ctx: &Context,
    template: &Template,
    digest: String,
    toolchain: String,
) -> Result<Verification> {
    let path = readiness_file(ctx, template, true)?;
    let evidence = Verification {
        at: tasks::now(),
        digest,
        toolchain,
        target: target_name(),
    };
    let staging = Stage::new(path.parent().context("Invalid verification directory")?)?;
    let file = staging.0.join("verification");
    fs::write(&file, serde_json::to_vec(&evidence)?)?;
    fs::rename(file, path)?;
    Ok(evidence)
}

pub(crate) fn bundle_source(ctx: &Context, reference: &str) -> Result<(Installed, PathBuf)> {
    let entry = select(ctx, reference, true)?;
    if let Some(path) = entry.installed {
        return Ok((verify_release(&path)?, path));
    }
    let path = source(ctx, &entry.template.path)?;
    let marker = Installed {
        template: entry.template,
        digest: tree_digest(&path)?,
        verified_at: None,
    };
    Ok((marker, path))
}

pub(crate) fn validate_portable(template: &Template) -> Result<()> {
    let (stack, _) = id_parts(&template.id)?;
    version(&template.version)?;
    let toolchain = match stack {
        "dotnet" => "dotnet",
        "elixir" => "elixir",
        "go" => "go",
        "java" => "mvn",
        "python" => "python3",
        "ruby" => "ruby",
        "rust" => "cargo",
        "swift" => "swift",
        "node" => "node",
        _ => bail!("Unsupported starter stack {stack}"),
    };
    if template.stack != stack
        || template.toolchain != toolchain
        || runtime_numbers(&template.minimum_version).is_none()
        || template
            .minimum_macos
            .as_deref()
            .is_some_and(|value| os_version(value).is_none())
        || template.test_command != "./test-offline.sh"
    {
        bail!("Invalid toolchain requirements for {}", template.id);
    }
    Ok(())
}

pub(crate) fn bundled_digest(ctx: &Context, template: &Template) -> Result<Option<String>> {
    ctx.catalog
        .templates
        .iter()
        .find(|entry| entry.id == template.id && entry.version == template.version)
        .map(|entry| tree_digest(&source(ctx, &entry.path)?))
        .transpose()
}

pub(crate) fn toolchain_report(template: &Template) -> Value {
    let (path, detail) = toolchain(template);
    let observed = path.as_ref().and_then(|_| {
        let (_, actual) = detail.split_once(" · ")?;
        let version = match template.toolchain.as_str() {
            "elixir" => actual.split("Elixir ").nth(1)?,
            "mvn" => actual.split("Java version: ").nth(1)?,
            "swift" => actual.split("Swift version ").nth(1)?,
            _ => actual,
        };
        Some(
            runtime_numbers(version)?
                .into_iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("."),
        )
    });
    json!({"available":path.is_some(),"path":path,"detail":detail,"version":observed})
}

pub fn remove(ctx: &Context, reference: &str) -> Result<Value> {
    let (id, label) = reference
        .split_once('@')
        .context("Select an exact starter release to remove")?;
    version(label)?;
    let _lock = lock_library(ctx)?;
    let base = installed_base(ctx, id, false)?;
    let path = base.join(label);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(json!({"id":id,"version":label,"removed":false}));
        }
        Ok(metadata) if metadata.is_dir() => {}
        _ => bail!("Expected an installed starter directory"),
    }
    let staging = Stage::new(&base)?;
    fs::rename(&path, staging.0.join("release"))?;
    let (stack, name) = id_parts(id)?;
    let readiness = library(ctx, &Path::new("readiness").join(stack).join(name), false)?;
    let _ = fs::remove_file(readiness.join(format!("{label}.json")));
    Ok(json!({"id":id,"version":label,"removed":true}))
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
    let _lock = if dry_run {
        None
    } else {
        Some(lock_library(ctx)?)
    };
    let entry = select(ctx, reference, true)?;
    let template = entry.template;
    let installed = entry.installed.is_some();
    let input = match entry.installed {
        Some(input) => input,
        None if dry_run => source(ctx, &template.path)?,
        None => bail!(
            "Install {}@{} before creating a project",
            template.id,
            template.version
        ),
    };
    if !target_matches(&template) {
        bail!("{}", target_detail(&template));
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
    let (tool, detail) = toolchain(&template);
    let tool = tool.context(detail.clone())?;
    let marker = installed.then(|| verify_release(&input)).transpose()?;
    let has_test_script = template.test_command == "./test-offline.sh"
        && regular(&input.join("project/test-offline.sh")).is_ok();
    let test_command = if has_test_script {
        "./devkit-test.sh"
    } else {
        &template.test_command
    };
    if !dry_run {
        let marker = marker.context("Install the selected starter first")?;
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
        drop(_lock);
        let _project_lock =
            crate::project_prepare::lock_generated(ctx, &destination, &template.stack)?;
        if fs::symlink_metadata(&destination).is_ok() {
            bail!("The destination already exists: {}", destination.display());
        }
        crate::project_prepare::verify_generated(
            ctx,
            &stage.0,
            &destination,
            &template.stack,
            &template.toolchain,
            &tool,
            &detail,
        )?;
        {
            let _lock = lock_library(ctx)?;
            if verify_release(&input).is_ok_and(|current| current.digest == marker.digest) {
                record_verification(ctx, &template, marker.digest, detail)?;
            }
        }
        stage.publish(&destination)?;
    }
    Ok(
        json!({"path":destination,"template":template.id,"version":template.version,"test_command":test_command,"toolchain_path":tool,"services":template.services,"docs":template.docs,"dry_run":dry_run}),
    )
}

pub(crate) fn verify_release(path: &Path) -> Result<Installed> {
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

pub(crate) fn run_command(
    command: &mut Command,
    label: &str,
    job: Option<&mut tasks::Job>,
) -> Result<()> {
    if let Some(job) = job {
        job.command(command, label)
    } else {
        completed(command, label)
    }
}

pub(crate) fn verify_build_job(
    project: &Path,
    work: &Path,
    tool: &Path,
    tool_name: &str,
    mut job: Option<&mut tasks::Job>,
) -> Result<()> {
    if let Some(job) = job.as_deref_mut() {
        job.stage("Preparing isolated offline test")?;
    }
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
    run_command(
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
        "Network-denied verification",
        job,
    )
}

pub(crate) fn verify_offline(
    ctx: &Context,
    reference: &str,
    mut job: Option<&mut tasks::Job>,
) -> Result<Value> {
    if let Some(job) = job.as_deref_mut() {
        job.stage("Checking installed starter and toolchain")?;
    }
    let (template, source, marker) = {
        let _lock = lock_library(ctx)?;
        let entry = select(ctx, reference, true)?;
        let source = entry
            .installed
            .context("Install this starter release before verification")?;
        let marker = verify_release(&source)?;
        if !target_matches(&entry.template) {
            bail!("{}", target_detail(&entry.template));
        }
        let file = readiness_file(ctx, &entry.template, false)?;
        if fs::symlink_metadata(&file).is_ok_and(|m| m.is_file()) {
            fs::remove_file(file)?;
        }
        (entry.template, source, marker)
    };
    let (tool, detail) = toolchain(&template);
    let tool = tool.context(detail.clone())?;
    let work = library(ctx, Path::new("verification"), true)?;
    let stage = Stage::new(&work)?;
    verify_build_job(
        &source.join("project"),
        &stage.0,
        &tool,
        &template.toolchain,
        job.as_deref_mut(),
    )?;
    if let Some(job) = job {
        job.stage("Recording offline readiness")?;
    }
    let _lock = lock_library(ctx)?;
    if verify_release(&source)?.digest != marker.digest {
        bail!("The starter changed while it was being verified");
    }
    let evidence = record_verification(ctx, &template, marker.digest, detail)?;
    Ok(
        json!({"ready_offline":true,"status":"ready_offline","template":template,"verification":evidence}),
    )
}

pub(crate) fn prepare(
    ctx: &Context,
    reference: &str,
    mut job: Option<&mut tasks::Job>,
) -> Result<Value> {
    if let Some(job) = job.as_deref_mut() {
        job.stage("Installing starter release")?;
    }
    let installed = install(ctx, reference)?;
    let label = format!(
        "{}@{}",
        text_arg(&installed, "id")?,
        text_arg(&installed, "version")?
    );
    let template = select(ctx, &label, true)?.template;
    let mut docs = Vec::new();
    for doc in &template.docs {
        if let Some(job) = job.as_deref_mut() {
            job.stage(&format!("Installing {doc} documentation"))?;
        }
        let (id, version) = doc
            .split_once('@')
            .map_or((doc.as_str(), None), |(id, version)| (id, Some(version)));
        docs.push(crate::install_docs(ctx, id, version)?);
    }
    verify_offline(ctx, &label, job)?;
    Ok(json!({"template":installed,"docs":docs,"doctor":doctor(ctx,&label)?}))
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
    build_latest_job(ctx, reference, None)
}

pub(crate) fn build_latest_job(
    ctx: &Context,
    reference: &str,
    mut job: Option<&mut tasks::Job>,
) -> Result<Value> {
    if let Some(job) = job.as_deref_mut() {
        job.stage("Checking starter recipe and toolchain")?;
    }
    let entry = select(ctx, reference, false)?;
    let mut template = entry.template;
    let catalogued = ctx
        .catalog
        .templates
        .iter()
        .any(|release| release.id == template.id && release.version == template.version);
    let source = if catalogued {
        source(ctx, &template.path)?
    } else if let Some(path) = entry.installed {
        verify_release(&path)?;
        path
    } else {
        bail!("Starter recipe source is unavailable");
    };
    if !target_matches(&template) {
        bail!("{}", target_detail(&template));
    }
    let (tool, detail) = toolchain(&template);
    let tool = tool.context(detail.clone())?;
    let base = installed_base(ctx, &template.id, true)?;
    let stage = Stage::new(&base)?;
    copy_tree(&source, &stage.0, None)?;
    if !catalogued {
        fs::remove_file(stage.0.join("installed.json"))?;
    }
    let recipe = stage.0.join("build-latest.sh");
    regular(&recipe).context("This template does not provide a latest-version recipe")?;
    let project = stage.0.join("project");
    let verification = Stage::new(&base)?;
    let mut builder = Command::new("/bin/sh");
    tool_environment(&mut builder, &template.toolchain, &tool)?;
    run_command(
        builder
            .arg(&recipe)
            .arg(&project)
            .current_dir(&stage.0)
            .env("TMPDIR", &verification.0)
            .env("DEVKIT_TOOLCHAIN", &tool),
        "Refreshing upstream dependencies",
        job.as_deref_mut(),
    )?;
    verify_build_job(
        &project,
        &verification.0,
        &tool,
        &template.toolchain,
        job.as_deref_mut(),
    )?;
    if let Some(job) = job.as_deref_mut() {
        job.stage("Publishing verified starter release")?;
    }
    let _lock = lock_library(ctx)?;
    if let Some(job) = job {
        job.checkpoint()?;
    }
    template.version = next_release_version(ctx, &template)?;
    let output = base.join(&template.version);
    template.verified = true;
    template.description = format!(
        "Locally rebuilt {} with resolved and offline-tested dependencies.",
        template.name
    );
    let coverage_file = project.join("dependency-coverage.json");
    if regular(&coverage_file).is_ok() {
        let mut coverage: Value = serde_json::from_slice(&fs::read(&coverage_file)?)?;
        coverage["starter"] = json!(format!("{}@{}", template.id, template.version));
        for field in ["vendored_modules", "vendored_crates"] {
            if coverage.get(field).is_some() {
                coverage[field] = json!(dependency_inventory(&project, &template.stack)?);
            }
        }
        let vendor = project.join("vendor");
        if vendor.is_dir() {
            coverage["vendor_bytes"] = json!(release_size(&vendor)?.1);
        }
        fs::write(coverage_file, serde_json::to_vec_pretty(&coverage)?)?;
    }
    let digest = tree_digest(&stage.0)?;
    record_verification(ctx, &template, digest, detail)?;
    let result = publish_release(&stage, template.clone(), &output, Some(tasks::now()));
    if result.is_err() {
        let _ = fs::remove_file(readiness_file(ctx, &template, false)?);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swift_version_uses_compiler_version_and_preserves_driver_alias() -> Result<()> {
        let fixture = Fixture::new()?;
        let mut template = fixture.ctx.catalog.templates[0].clone();
        template.toolchain = "swift".into();
        template.minimum_version = "6.0".into();
        let observed = "swift-driver version: 1.168.6 Apple Swift version 6.4.2 (swiftlang) Target: arm64-apple-macosx27.2";
        assert!(meets_minimum(&template, observed));
        template.minimum_version = "6.5".into();
        assert!(!meets_minimum(&template, observed));
        #[cfg(unix)]
        {
            let frontend = fixture._root.0.join("swift-frontend");
            let driver = fixture._root.0.join("swift");
            fs::copy("/usr/bin/true", &frontend)?;
            std::os::unix::fs::symlink(&frontend, &driver)?;
            assert_eq!(known_executable(&driver, "swift"), Some(driver));
            assert_eq!(known_executable(Path::new("/usr/bin/swift"), "swift"), None);
        }
        Ok(())
    }
    use crate::Catalog;

    #[cfg(unix)]
    #[test]
    fn deeply_nested_release_keeps_its_digest_on_a_small_worker_stack() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let root = Stage::new(&env::temp_dir().canonicalize()?)?;
        let mut nested = root.0.join("vendor");
        for level in 0..20 {
            nested.push(format!("level-{level:02}"));
        }
        fs::create_dir_all(nested.join("empty"))?;
        let payload: Vec<_> = (0..70_123).map(|index| (index % 251) as u8).collect();
        for (file, bytes, mode) in [
            (nested.join("payload.bin"), payload.as_slice(), 0o644),
            (root.0.join("notes.txt"), b"starter\n".as_slice(), 0o644),
            (root.0.join("bin"), b"test\n".as_slice(), 0o755),
        ] {
            fs::write(&file, bytes)?;
            fs::set_permissions(file, fs::Permissions::from_mode(mode))?;
        }
        fs::write(root.0.join("installed.json"), "ignored metadata")?;

        let path = root.0.clone();
        let digest = std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(move || tree_digest(&path))?
            .join()
            .map_err(|_| anyhow::anyhow!("Template digest worker panicked"))??;
        // This value uses the original path, kind, mode, length, and byte encoding.
        assert_eq!(
            digest,
            "9ca9c8dc28491efaf603ddc0aa2b81c5e07778b572e7f4f5b169e3cf758b073b"
        );
        Ok(())
    }

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
                fs::write(
                    project.join("test-offline.sh"),
                    "#!/bin/sh\nset -eu\ntest -s main.txt\n",
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
                    minimum_macos: None,
                    test_command: "./test-offline.sh".into(),
                    docs: Vec::new(),
                    services: Vec::new(),
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
        publish_release(&stage, local, &base.join("1.0.5"), None)?;
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
