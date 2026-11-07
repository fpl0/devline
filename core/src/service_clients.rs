//! Explicit, transactional client dependency selection. Service images alone
//! never enter this path; the caller must provide concrete client IDs.
use crate::{
    project, project_prepare, safe_catalog_path, tasks::Job, templates, text_arg, Context,
};
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "service_client_artifacts.rs"]
mod artifacts;

const STACKS: &[&str] = &[
    "dotnet", "elixir", "go", "java", "node", "python", "ruby", "rust", "swift",
];
const SERVICES: &[&str] = &[
    "postgres",
    "mysql",
    "rabbitmq",
    "redis",
    "mongodb",
    "nats",
    "mailpit",
    "meilisearch",
    "seaweedfs",
    "keycloak",
];
fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map_or(&[], Vec::as_slice)
}
fn read(path: &Path) -> Option<String> {
    let meta = fs::symlink_metadata(path).ok()?;
    (meta.is_file() && meta.len() <= 4 * 1024 * 1024)
        .then(|| fs::read_to_string(path).ok())
        .flatten()
}
fn normal(name: &str) -> String {
    name.split('[')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
        .replace(['_', '.'], "-")
}
fn declarations(ctx: &Context, stack: &str, root: &Path) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let mut add = |name: &str| {
        names.insert(normal(name));
    };
    match stack {
        "node" => {
            if let Some(value) = read(&root.join("package.json"))
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            {
                for key in ["dependencies", "devDependencies", "optionalDependencies"] {
                    if let Some(deps) = value[key].as_object() {
                        for name in deps.keys() {
                            add(name);
                        }
                    }
                }
            }
        }
        "rust" => {
            if let Some(value) =
                read(&root.join("Cargo.toml")).and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            {
                for section in ["dependencies"] {
                    if let Some(deps) = value.get(section).and_then(toml::Value::as_table) {
                        for (name, entry) in deps {
                            add(entry
                                .get("package")
                                .and_then(toml::Value::as_str)
                                .unwrap_or(name));
                        }
                    }
                }
            }
        }
        "python" => {
            for file in ["requirements.txt", "requirements.in", "requirements.lock"] {
                if let Some(text) = read(&root.join(file)) {
                    for line in text.lines() {
                        let line = line.trim();
                        if line.starts_with(['#', '-']) {
                            continue;
                        }
                        let name = line
                            .split(|c: char| !(c.is_ascii_alphanumeric() || "_.-".contains(c)))
                            .next()
                            .unwrap_or_default();
                        if !name.is_empty() {
                            add(name);
                        }
                    }
                }
            }
            if let Some(value) = read(&root.join("pyproject.toml"))
                .and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            {
                if let Some(deps) = value
                    .get("project")
                    .and_then(|v| v.get("dependencies"))
                    .and_then(toml::Value::as_array)
                {
                    for dep in deps.iter().filter_map(toml::Value::as_str) {
                        add(dep
                            .split(|c: char| !(c.is_ascii_alphanumeric() || "_.-".contains(c)))
                            .next()
                            .unwrap_or_default());
                    }
                }
            }
        }
        "dotnet" => {
            if let Ok(entries) = fs::read_dir(root) {
                for entry in entries.flatten() {
                    if matches!(
                        entry.path().extension().and_then(|s| s.to_str()),
                        Some("csproj" | "fsproj" | "vbproj")
                    ) {
                        if let Some(text) = read(&entry.path()) {
                            if let Ok(doc) = roxmltree::Document::parse(&text) {
                                for node in doc
                                    .descendants()
                                    .filter(|n| n.tag_name().name() == "PackageReference")
                                {
                                    if let Some(name) = node
                                        .attribute("Include")
                                        .or_else(|| node.attribute("Update"))
                                    {
                                        add(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        "java" => {
            if let Some(text) = read(&root.join("pom.xml")) {
                if let Ok(doc) = roxmltree::Document::parse(&text) {
                    for node in doc.descendants().filter(|n| {
                        n.tag_name().name() == "dependency"
                            && n.parent().is_some_and(|p| {
                                p.tag_name().name() == "dependencies"
                                    && p.parent().is_some_and(|p| p.tag_name().name() == "project")
                            })
                    }) {
                        let value = |name| {
                            node.children()
                                .find(|n| n.tag_name().name() == name)
                                .and_then(|n| n.text())
                        };
                        if let (Some(group), Some(artifact)) =
                            (value("groupId"), value("artifactId"))
                        {
                            add(&format!("{group}:{artifact}"));
                        }
                    }
                }
            }
        }
        "go" => {
            if let Some(text) = read(&root.join("go.mod")) {
                for line in text.lines() {
                    let line = line.trim().strip_prefix("require ").unwrap_or(line.trim());
                    if let Some(name) = line
                        .split_whitespace()
                        .next()
                        .filter(|name| name.contains('/'))
                    {
                        add(name);
                    }
                }
            }
        }
        "ruby" => {
            if let Some(text) = read(&root.join("Gemfile")) {
                for line in text
                    .lines()
                    .map(str::trim)
                    .filter(|line| line.starts_with("gem ") || line.starts_with("gem("))
                {
                    if let Some(name) = line.split(['\'', '"']).nth(1) {
                        add(name);
                    }
                }
            }
        }
        "elixir" => return elixir_declarations(ctx, root),
        "swift" => return swift_declarations(ctx, root),
        _ => {}
    }
    Ok(names)
}

fn elixir_declarations(ctx: &Context, root: &Path) -> Result<BTreeSet<String>> {
    let reports = templates::project_runtime_reports(ctx, root, &["elixir".to_owned()])?;
    let Some(report) = reports
        .as_array()
        .and_then(|reports| reports.first())
        .filter(|report| report["available"] == true)
    else {
        return Ok(BTreeSet::new());
    };
    let tool = Path::new(text_arg(report, "path")?);
    // Parse, never execute, mix.exs. Keyword and tuple dependency literals have
    // the same AST; comments and strings cannot masquerade as declarations.
    let code = r#"ast = Code.string_to_quoted!(File.read!(hd(System.argv()))); {_, names} = Macro.prewalk(ast, [], fn
      {name, requirement} = node, names when is_atom(name) and (is_binary(requirement) or is_list(requirement)) -> {node, [Atom.to_string(name) | names]}
      {name, requirement, options} = node, names when is_atom(name) and is_binary(requirement) and is_list(options) -> {node, [Atom.to_string(name) | names]}
      node, names -> {node, names}
    end); names |> Enum.uniq() |> Enum.each(&IO.puts/1)"#;
    let mut command = Command::new(tool);
    command
        .args(["-e", code])
        .arg(root.join("mix.exs"))
        .env_clear();
    templates::project_tool_environment(&mut command, text_arg(report, "toolchain")?, tool)?;
    let output = command
        .output()
        .context("Could not inspect Mix dependency declarations")?;
    if !output.status.success() {
        bail!(
            "Could not inspect Mix dependency declarations: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?
        .lines()
        .map(normal)
        .collect())
}

fn swift_target_products(description: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for dependency in rows(description, "dependencies") {
        for source in rows(dependency, "sourceControl") {
            if let Some(identity) = source["identity"].as_str() {
                names.insert(format!("package:{}", normal(identity)));
            }
        }
    }
    let target = rows(description, "targets")
        .iter()
        .find(|target| target["type"] == "executable")
        .or_else(|| {
            rows(description, "targets")
                .iter()
                .find(|target| target["type"] == "regular")
        });
    if let Some(target) = target {
        for dependency in rows(target, "dependencies") {
            let product = rows(dependency, "product");
            if let (Some(product), Some(package)) = (
                product.first().and_then(Value::as_str),
                product.get(1).and_then(Value::as_str),
            ) {
                names.insert(format!("{}:{product}", normal(package)));
            } else if let Some(name) = rows(dependency, "byName").first().and_then(Value::as_str) {
                names.insert(format!("*:{name}"));
            }
        }
    }
    names
}

fn swift_declarations(ctx: &Context, root: &Path) -> Result<BTreeSet<String>> {
    let reports = templates::project_runtime_reports(ctx, root, &["swift".to_owned()])?;
    let report = reports.as_array().and_then(|reports| reports.first());
    let Some(report) = report.filter(|report| report["available"] == true) else {
        return Ok(BTreeSet::new());
    };
    let tool = Path::new(text_arg(report, "path")?);
    let work = templates::Stage::new(&std::env::temp_dir().canonicalize()?)?;
    let mut command = Command::new("/usr/bin/sandbox-exec");
    let writable = serde_json::to_string(&work.0)?;
    command.args(["-p", &format!("(version 1)(allow default)(deny network-outbound)(deny file-write*)(allow file-write* (subpath {writable}) (literal \"/dev/null\"))")])
        .arg(tool).args(["package", "--disable-sandbox", "--manifest-cache", "local"])
        .arg("--scratch-path").arg(work.0.join("build"))
        .arg("--cache-path").arg(work.0.join("cache"))
        .arg("--config-path").arg(work.0.join("config"))
        .arg("--security-path").arg(work.0.join("security"))
        .arg("dump-package").current_dir(root).env_clear()
        .env("HOME", &work.0).env("TMPDIR", &work.0)
        .env("CLANG_MODULE_CACHE_PATH", work.0.join("clang-cache"));
    templates::project_tool_environment(&mut command, text_arg(report, "toolchain")?, tool)?;
    let output = command
        .output()
        .context("Could not inspect Swift target dependencies")?;
    if !output.status.success() {
        bail!(
            "Could not inspect Swift target dependencies: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(swift_target_products(&serde_json::from_slice(
        &output.stdout,
    )?))
}

fn package_present(
    stack: &str,
    root: Option<&Path>,
    names: &BTreeSet<String>,
    package: &Value,
) -> bool {
    let name = package["name"].as_str().unwrap_or_default();
    if stack == "swift" {
        return names.contains(&format!("package:{}", normal(name)))
            && rows(package, "products")
                .iter()
                .filter_map(Value::as_str)
                .all(|product| {
                    names.contains(&format!("{}:{product}", normal(name)))
                        || names.contains(&format!("*:{product}"))
                });
    }
    if !names.contains(&normal(name)) {
        return false;
    }
    if stack != "rust" || rows(package, "features").is_empty() {
        return true;
    }
    let manifest = root
        .and_then(|root| read(&root.join("Cargo.toml")))
        .and_then(|text| toml::from_str::<toml::Value>(&text).ok());
    let Some(manifest) = manifest else {
        return false;
    };
    ["dependencies"].iter().any(|section| {
        manifest
            .get(section)
            .and_then(toml::Value::as_table)
            .is_some_and(|dependencies| {
                dependencies.iter().any(|(key, value)| {
                    let id = value
                        .get("package")
                        .and_then(toml::Value::as_str)
                        .unwrap_or(key);
                    normal(id) == normal(name)
                        && rows(package, "features").iter().all(|feature| {
                            value
                                .get("features")
                                .and_then(toml::Value::as_array)
                                .is_some_and(|features| {
                                    features
                                        .iter()
                                        .any(|value| value.as_str() == feature.as_str())
                                })
                        })
                })
            })
    })
}
fn definitions(ctx: &Context, stack: &str) -> Result<Vec<Value>> {
    if !STACKS.contains(&stack) {
        bail!("Choose one supported project runtime for client libraries");
    }
    let file = safe_catalog_path(ctx, &format!("service-clients/{stack}.json"))?;
    let value: Value = serde_json::from_slice(&fs::read(file)?)?;
    let rows = rows(&value, "integrations");
    let mut found = BTreeSet::new();
    for item in rows {
        let service = text_arg(item, "service")?;
        if !SERVICES.contains(&service)
            || item["id"] != format!("{stack}:{service}")
            || !found.insert(service)
        {
            bail!("Invalid service client catalog identity");
        }
        if !text_arg(item, "source_url")?.starts_with("https://") {
            bail!("Client recommendation has no official source");
        }
        for package in rows_of_packages(item) {
            text_arg(package, "name")?;
            text_arg(package, "version")?;
        }
    }
    if found.len() != SERVICES.len() {
        bail!("Service client catalog does not cover every supported service");
    }
    Ok(rows.to_vec())
}
fn rows_of_packages(item: &Value) -> &[Value] {
    rows(item, "packages")
}
fn target(ctx: &Context, request: &Value) -> Result<(String, Option<PathBuf>)> {
    if let Some(path) = request["path"].as_str() {
        let profile = project::context(ctx, Path::new(path))?;
        let stack = profile["stack"]
            .as_str()
            .context("Open a project with one supported runtime manifest")?;
        if request["stack"]
            .as_str()
            .is_some_and(|selected| selected != stack)
        {
            bail!("Selected runtime does not match this project");
        }
        return Ok((
            stack.to_owned(),
            Some(PathBuf::from(text_arg(&profile, "path")?)),
        ));
    }
    if let Some(reference) = request["id"].as_str() {
        let (installed, path) = templates::bundle_source(ctx, reference)?;
        return Ok((installed.template.stack, Some(path.join("project"))));
    }
    Ok((text_arg(request, "stack")?.to_owned(), None))
}
pub(crate) fn catalog(ctx: &Context, request: &Value) -> Result<Value> {
    let (stack, path) = target(ctx, request)?;
    let declared = path
        .as_ref()
        .map(|path| declarations(ctx, &stack, path))
        .transpose()?
        .unwrap_or_default();
    let ready = safe_catalog_path(
        ctx,
        &format!("service-clients/artifacts/{stack}/ready.json"),
    )
    .ok()
    .and_then(|path| read(&path))
    .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let pack_available = ready
        .as_ref()
        .is_some_and(|ready| artifacts::check(ctx, &stack, ready, false).is_ok());
    let project_ready = path.as_ref().is_some_and(|path| {
        project_prepare::readiness(ctx, &json!({"path":path}))
            .is_ok_and(|proof| proof["ready_offline"] == true)
    });
    let mut services = Vec::new();
    for mut item in definitions(ctx, &stack)? {
        let packages = rows_of_packages(&item);
        let builtin = item["builtin"] == true;
        let installed = builtin
            || !packages.is_empty()
                && packages
                    .iter()
                    .all(|p| package_present(&stack, path.as_deref(), &declared, p));
        let available = builtin
            || installed && project_ready
            || pack_available
                && ready.as_ref().is_some_and(|ready| {
                    packages.iter().all(|p| {
                        rows(ready, "packages").iter().any(|cached| {
                            cached["name"] == p["name"] && cached["version"] == p["version"]
                        })
                    })
                });
        item["installed"] = json!(installed);
        item["offline_available"] = json!(available);
        item["supported"] = json!(item["supported"] != false);
        item["version"] = json!(packages_version(&item));
        let id = item["service"].clone();
        services.push(json!({"id":id,"name":id,"libraries":[item]}));
    }
    Ok(json!({"stack":stack,"services":services,"selection_required":true}))
}
fn packages_version(item: &Value) -> String {
    rows_of_packages(item)
        .iter()
        .filter_map(|p| p["version"].as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
fn stage(job: &mut Option<&mut Job>, label: &str) -> Result<()> {
    if let Some(job) = job.as_deref_mut() {
        job.stage(label)?;
    }
    Ok(())
}

fn locked_versions(stack: &str, root: &Path) -> Result<std::collections::BTreeMap<String, String>> {
    use std::collections::BTreeMap;
    let mut versions = BTreeMap::new();
    match stack {
        "dotnet" => {
            if let Some(text) = read(&root.join("packages.lock.json")) {
                let lock: Value = serde_json::from_str(&text)?;
                if let Some(frameworks) = lock["dependencies"].as_object() {
                    for (framework, packages) in frameworks {
                        if let Some(packages) = packages.as_object() {
                            for (name, package) in packages {
                                if let Some(version) = package["resolved"].as_str() {
                                    versions.insert(format!("{framework}/{name}"), version.into());
                                }
                            }
                        }
                    }
                }
            }
        }
        "node" => {
            if let Some(text) = read(&root.join("package-lock.json")) {
                let lock: Value = serde_json::from_str(&text)?;
                if let Some(packages) = lock["packages"].as_object() {
                    for (name, package) in packages.iter().filter(|(name, _)| !name.is_empty()) {
                        if let Some(version) = package["version"].as_str() {
                            versions.insert(name.clone(), version.into());
                        }
                    }
                }
            }
        }
        "rust" => {
            if let Some(text) = read(&root.join("Cargo.lock")) {
                let lock: toml::Value = toml::from_str(&text)?;
                if let Some(packages) = lock.get("package").and_then(toml::Value::as_array) {
                    for package in packages {
                        if let (Some(name), Some(version), Some(source)) = (
                            package.get("name").and_then(toml::Value::as_str),
                            package.get("version").and_then(toml::Value::as_str),
                            package.get("source").and_then(toml::Value::as_str),
                        ) {
                            // Cargo permits multiple versions of one crate. Keep
                            // each existing identity, including Git revisions.
                            versions.insert(format!("{name}@{version}"), source.into());
                        }
                    }
                }
            }
        }
        "go" => {
            if let Some(text) = read(&root.join("go.mod")) {
                let mut block = false;
                for line in text.lines().map(str::trim) {
                    if line.starts_with("require (") {
                        block = true;
                        continue;
                    }
                    if block && line == ")" {
                        block = false;
                        continue;
                    }
                    let requirement = if block {
                        Some(line)
                    } else {
                        line.strip_prefix("require ")
                    };
                    if let Some(requirement) = requirement {
                        let mut parts = requirement.split_whitespace();
                        if let (Some(name), Some(version)) = (parts.next(), parts.next()) {
                            if version.starts_with('v') {
                                versions.insert(name.into(), version.into());
                            }
                        }
                    }
                }
            }
        }
        "python" => {
            for file in ["requirements.lock", "requirements.txt", "requirements.in"] {
                if let Some(text) = read(&root.join(file)) {
                    for line in text
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.starts_with('#'))
                    {
                        if let Some((name, rest)) = line.split_once("==") {
                            if !name.contains([' ', ';']) {
                                let version =
                                    rest.split([' ', ';', '\\']).next().unwrap_or_default();
                                versions.insert(normal(name), version.into());
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(versions)
}

fn preserve_locked_versions(stack: &str, original: &Path, staged: &Path) -> Result<()> {
    let previous = locked_versions(stack, original)?;
    if previous.is_empty() {
        return Ok(());
    }
    let next = locked_versions(stack, staged)?;
    for (name, version) in previous {
        if next.get(&name) != Some(&version) {
            bail!("Adding the selected client would change the existing lock for {name} ({version}). Update that dependency explicitly first; the original project was left unchanged.");
        }
    }
    Ok(())
}

fn changed_paths(stack: &str, copy: &Path, original: &Path) -> Result<Vec<PathBuf>> {
    fn walk(
        stack: &str,
        copy: &Path,
        original: &Path,
        relative: &Path,
        result: &mut Vec<PathBuf>,
    ) -> Result<()> {
        for entry in fs::read_dir(copy.join(relative))? {
            let entry = entry?;
            let part = relative.join(entry.file_name());
            let name = entry.file_name();
            let name = name.to_str().context("Non-UTF-8 project path")?;
            let dependency = relative.as_os_str().is_empty()
                && matches!(
                    name,
                    ".devline-offline"
                        | "vendor"
                        | "node_modules"
                        | "wheelhouse"
                        | "nuget-packages"
                        | "maven-repository"
                        | "hex-cache"
                );
            if !dependency
                && (matches!(name, ".git" | ".devline-infra" | ".DS_Store")
                    || name.starts_with(".devkit-") && name != ".devkit-origin.json"
                    || match stack {
                        "dotnet" => matches!(name, "bin" | "obj" | "TestResults"),
                        "rust" | "java" => name == "target",
                        "swift" => name == ".build",
                        "elixir" => matches!(name, "_build" | "deps"),
                        "python" => matches!(name, ".venv" | "__pycache__" | ".pytest_cache"),
                        _ => false,
                    })
            {
                continue;
            }
            let source = copy.join(&part);
            let target = original.join(&part);
            let meta = fs::symlink_metadata(&source)?;
            if meta.is_dir() {
                if dependency || !target.exists() {
                    result.push(part);
                } else if target.is_dir() {
                    walk(stack, copy, original, &part, result)?;
                } else {
                    bail!("Client dependency change would replace a source file with a directory");
                }
            } else if meta.is_file() {
                let same = fs::symlink_metadata(&target)
                    .is_ok_and(|m| m.is_file() && m.len() == meta.len())
                    && fs::read(&target)? == fs::read(&source)?;
                if !same {
                    result.push(part);
                }
            } else {
                bail!("Client adapter created an unexpected symbolic link or special file");
            }
        }
        Ok(())
    }
    let mut result = Vec::new();
    walk(stack, copy, original, Path::new(""), &mut result)?;
    result.sort();
    Ok(result)
}
#[derive(serde::Serialize, serde::Deserialize)]
struct MutationChange {
    relative: PathBuf,
    had_original: bool,
    digest: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Mutation {
    project: PathBuf,
    work: PathBuf,
    #[serde(default)]
    committed: bool,
    changes: Vec<MutationChange>,
}
fn journal_path(ctx: &Context, path: &Path) -> Result<PathBuf> {
    use sha2::{Digest, Sha256};
    let root = templates::library(ctx, Path::new("client-mutations"), true)?;
    Ok(root.join(format!(
        "{:x}.json",
        Sha256::digest(path.as_os_str().as_encoded_bytes())
    )))
}
fn digest_path(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    fn visit(path: &Path, digest: &mut Sha256) -> Result<()> {
        let meta = fs::symlink_metadata(path)?;
        if meta.is_symlink() {
            digest.update(b"l");
            digest.update(fs::read_link(path)?.as_os_str().as_encoded_bytes());
        } else if meta.is_dir() {
            digest.update(b"d");
            let mut entries: Vec<_> = fs::read_dir(path)?.collect::<std::io::Result<_>>()?;
            entries.sort_by_key(fs::DirEntry::file_name);
            for entry in entries {
                digest.update(entry.file_name().as_encoded_bytes());
                digest.update([0]);
                visit(&entry.path(), digest)?;
            }
        } else if meta.is_file() {
            digest.update(b"f");
            let mut file = fs::File::open(path)?;
            let mut buffer = [0u8; 65536];
            loop {
                let size = file.read(&mut buffer)?;
                if size == 0 {
                    break;
                }
                digest.update(&buffer[..size]);
            }
        } else {
            bail!("Unexpected special file in client dependency change");
        }
        Ok(())
    }
    let mut digest = Sha256::new();
    visit(path, &mut digest)?;
    Ok(format!("{:x}", digest.finalize()))
}
fn save_journal(ctx: &Context, mutation: &Mutation) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let path = journal_path(ctx, &mutation.project)?;
    let temporary = templates::Stage::new(path.parent().unwrap())?;
    let file = temporary.0.join("journal");
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&file)?;
    serde_json::to_writer(&mut output, mutation)?;
    output.sync_all()?;
    fs::rename(file, &path)?;
    fs::File::open(path.parent().unwrap())?.sync_all()?;
    Ok(())
}
/// Caller holds the same project mutation lock as preparation and client adds.
pub(crate) fn recover(ctx: &Context, project: &Path) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let path = journal_path(ctx, project)?;
    let input = match fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&path)
    {
        Ok(input) => input,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !input.metadata()?.is_file() || input.metadata()?.len() > 16 * 1024 * 1024 {
        bail!("Invalid pending client dependency transaction");
    }
    let mutation: Mutation = serde_json::from_reader(input)?;
    if mutation.project != project
        || mutation.work.parent() != project.parent()
        || !mutation
            .work
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(".devkit-"))
        || !fs::symlink_metadata(&mutation.work)?.is_dir()
        || mutation.changes.iter().any(|change| {
            change
                .relative
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
                || change.relative.starts_with(".git")
                || change.relative.starts_with(".devline-infra")
        })
    {
        bail!("Pending client dependency transaction has an invalid path");
    }
    if !mutation.committed {
        for (index, change) in mutation.changes.iter().enumerate().rev() {
            let target = project.join(&change.relative);
            let source = mutation.work.join("project").join(&change.relative);
            let backup = mutation.work.join(format!("backup-{index}"));
            let was_published = !source.try_exists()? && target.try_exists()?;
            if was_published {
                if digest_path(&target)? != change.digest {
                    bail!("A project file changed after interrupted client preparation. Recovery copies remain at {}",mutation.work.display());
                }
                if let Some(parent) = source.parent() {
                    fs::create_dir_all(parent)?;
                }
                templates::rename_exclusive(&target, &source)?;
            }
            if change.had_original && backup.try_exists()? {
                templates::rename_exclusive(&backup, &target)?;
            }
        }
    }
    fs::remove_file(&path)?;
    fs::File::open(path.parent().unwrap())?.sync_all()?;
    let _ = fs::remove_dir_all(&mutation.work);
    Ok(())
}

pub(crate) fn add(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let selections = request["clients"]
        .as_array()
        .context("Choose client libraries explicitly")?;
    if selections.is_empty() {
        return Ok(json!({"changed":false,"complete":true,"status":"unchanged","clients":[]}));
    }
    if selections.len() > SERVICES.len() {
        bail!("Choose at most one client recommendation per service");
    }
    let path = Path::new(text_arg(request, "path")?).canonicalize()?;
    let _lock = project_prepare::lock_project(ctx, &json!({"path":path}), &job)?;
    recover(ctx, &path)?;
    if path.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a project outside the Devline library");
    }
    let (stack, _) = target(ctx, &json!({"path":path}))?;
    let declared = declarations(ctx, &stack, &path)?;
    let definitions = definitions(ctx, &stack)?;
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for selection in selections {
        let id = selection
            .as_str()
            .context("Client selections must be stable catalog IDs")?;
        if !seen.insert(id) {
            bail!("Client selected more than once: {id}");
        }
        let mut item = definitions
            .iter()
            .find(|item| item["id"] == id)
            .cloned()
            .with_context(|| {
                format!("Client {id} is not recommended for this project's {stack} runtime")
            })?;
        if item["supported"] == false {
            bail!(
                "{}",
                item["detail"]
                    .as_str()
                    .unwrap_or("This client is not supported")
            );
        }
        let mut missing: Vec<_> = rows_of_packages(&item)
            .iter()
            .filter(|p| !package_present(&stack, Some(&path), &declared, p))
            .cloned()
            .collect();
        if stack == "rust" {
            if let Some(manifest) =
                read(&path.join("Cargo.toml")).and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            {
                for package in &mut missing {
                    let package_name = package["name"].as_str().unwrap_or_default();
                    if let Some(dependencies) =
                        manifest.get("dependencies").and_then(toml::Value::as_table)
                    {
                        if let Some((name, value)) = dependencies.iter().find(|(name, value)| {
                            value
                                .get("package")
                                .and_then(toml::Value::as_str)
                                .unwrap_or(name)
                                == package_name
                        }) {
                            if let Some(requirement) = value
                                .as_str()
                                .or_else(|| value.get("version").and_then(toml::Value::as_str))
                            {
                                package["existing_requirement"] = json!(requirement);
                                package["existing_name"] = json!(name);
                            }
                        }
                    }
                }
            }
        }
        item["packages"] = json!(missing);
        if !rows_of_packages(&item).is_empty() {
            selected.push(item);
        }
    }
    if selected.is_empty() {
        return Ok(
            json!({"changed":false,"complete":true,"status":"already_included","clients":selections}),
        );
    }
    stage(&mut job, "Verifying prepared client artifacts")?;
    let marker = safe_catalog_path(
        ctx,
        &format!("service-clients/artifacts/{stack}/ready.json"),
    )?;
    let ready: Value = serde_json::from_slice(&fs::read(marker)?)?;
    artifacts::check(ctx, &stack, &ready, true).context("Prepared client pack is incomplete or changed; restore the bundled pack before adding clients")?;
    stage(&mut job, "Staging selected client dependency changes")?;
    let work = templates::Stage::new(path.parent().context("Project has no parent")?)?;
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&work.0, fs::Permissions::from_mode(0o700))?;
    }
    let copy = work.0.join("project");
    fs::create_dir(&copy)?;
    let before = project_prepare::mutation_snapshot(&stack, &path, Some(&copy))?;
    let reports = templates::project_runtime_reports(ctx, &path, std::slice::from_ref(&stack))?;
    let report = reports
        .as_array()
        .and_then(|r| r.first())
        .context("No runtime report")?;
    if report["available"] != true || report["compatible"] == false {
        bail!(
            "{}",
            report["detail"]
                .as_str()
                .unwrap_or("Required local runtime is unavailable")
        );
    }
    let tool = Path::new(text_arg(report, "path")?);
    let adapter = safe_catalog_path(ctx, &format!("service-clients/{stack}.sh"))?;
    let selection = work.0.join("selection.json");
    let offline = request["offline"] == true;
    fs::write(
        &selection,
        serde_json::to_vec(
            &json!({"stack":stack,"integrations":selected,"catalog_dir":ctx.catalog_dir.canonicalize()?,"offline":offline}),
        )?,
    )?;
    let home = work.0.join("home");
    let tmp = work.0.join("tmp");
    fs::create_dir(&home)?;
    fs::create_dir(&tmp)?;
    let mut command = Command::new("/usr/bin/sandbox-exec");
    let writable = serde_json::to_string(&work.0)?;
    let network = if offline {
        "(deny network-outbound)(allow network-outbound (remote ip \"localhost:*\"))"
    } else {
        ""
    };
    let platform = if stack == "dotnet" {
        "(allow file-write* (subpath \"/private/tmp/.dotnet/shm\") (subpath \"/private/tmp/.dotnet/lockfiles\") (literal \"/private/tmp/.dotnet\"))"
    } else {
        ""
    };
    command.args(["-p",&format!("(version 1)(allow default)(deny file-write*)(allow file-write* (subpath {writable}) (literal \"/dev/null\")){platform}{network}"),"/bin/sh"])
        .arg(&adapter).arg(&copy).arg(&selection).arg(if offline {"true"} else {"false"}).arg(tool).current_dir(&copy)
        .env_clear().env("HOME",&home).env("TMPDIR",&tmp).env("DEVLINE_OFFLINE",if offline {"true"}else{"false"});
    templates::project_tool_environment(&mut command, text_arg(report, "toolchain")?, tool)?;
    templates::run_command(
        &mut command,
        "Adding and locking selected client libraries",
        job.as_deref_mut(),
    )?;
    preserve_locked_versions(&stack, &path, &copy)?;
    stage(
        &mut job,
        "Verifying the changed project with external networking denied",
    )?;
    project_prepare::prepare(
        ctx,
        &json!({"path":copy,"offline":true,"force_adapter":true}),
        job.as_deref_mut(),
    )?;
    stage(&mut job, "Publishing verified client dependencies")?;
    if project_prepare::mutation_snapshot(&stack, &path, None)? != before {
        bail!(
            "Project changed while preparing client libraries; original files were left untouched"
        );
    }
    let changes = changed_paths(&stack, &copy, &path)?
        .into_iter()
        .map(|relative| {
            Ok(MutationChange {
                had_original: fs::symlink_metadata(path.join(&relative)).is_ok(),
                digest: digest_path(&copy.join(&relative))?,
                relative,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut mutation = Mutation {
        project: path.clone(),
        work: work.0.clone(),
        committed: false,
        changes,
    };
    save_journal(ctx, &mutation)?;
    let result = (|| -> Result<Value> {
        for (index, change) in mutation.changes.iter().enumerate() {
            let source = copy.join(&change.relative);
            let target = path.join(&change.relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            if change.had_original {
                templates::rename_exclusive(&target, &work.0.join(format!("backup-{index}")))?;
            }
            templates::rename_exclusive(&source, &target)?;
        }
        let readiness = project_prepare::record_mutation(ctx, &path, &copy)?;
        // A durable commit marker distinguishes this operation from an older
        // proof with the same content fingerprint. Until this is saved, recovery
        // conservatively restores the original even if proof recording finished.
        mutation.committed = true;
        save_journal(ctx, &mutation)?;
        Ok(
            json!({"changed":true,"complete":true,"status":"ready_offline","ready_offline":true,"clients":selections,"readiness":readiness}),
        )
    })();
    if let Err(error) = recover(ctx, &path) {
        let recovery = work.0.clone();
        std::mem::forget(work);
        return Err(error).with_context(|| {
            format!(
                "Client transaction recovery files remain at {}",
                recovery.display()
            )
        });
    }
    result
}

#[cfg(test)]
mod transaction_tests {
    use super::*;

    fn fixture() -> Result<(templates::Stage, Context, PathBuf)> {
        let root = templates::Stage::new(&std::env::temp_dir().canonicalize()?)?;
        let home = root.0.join("home");
        let project = root.0.join("original");
        fs::create_dir(&home)?;
        fs::create_dir(&project)?;
        Ok((
            root,
            Context {
                home,
                catalog_dir: PathBuf::new(),
                catalog: crate::Catalog {
                    schema_version: 1,
                    docs: vec![],
                    templates: vec![],
                },
            },
            project,
        ))
    }

    #[test]
    fn empty_selection_never_reads_or_mutates_project() -> Result<()> {
        let (_root, ctx, project) = fixture()?;
        fs::write(project.join("untouched"), "original")?;
        let result = add(&ctx, &json!({"path":project,"clients":[]}), None)?;
        assert_eq!(result["changed"], false);
        assert_eq!(fs::read_dir(&project)?.count(), 1);
        assert_eq!(fs::read_to_string(project.join("untouched"))?, "original");
        Ok(())
    }

    #[test]
    fn publication_selects_only_changes_preserving_nested_ignored_files() -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        let (root, _ctx, original) = fixture()?;
        let copy = root.0.join("copy");
        fs::create_dir_all(original.join("src/__pycache__"))?;
        fs::create_dir_all(copy.join("src"))?;
        fs::write(original.join("src/__pycache__/local.pyc"), "private cache")?;
        fs::write(original.join("src/main.py"), "print('hello')\n")?;
        fs::write(copy.join("src/main.py"), "print('hello')\n")?;
        fs::write(original.join("requirements.txt"), "")?;
        fs::write(copy.join("requirements.txt"), "pika==1.4.4\n")?;
        let inode = fs::metadata(original.join("src/main.py"))?.ino();
        assert_eq!(
            changed_paths("python", &copy, &original)?,
            vec![PathBuf::from("requirements.txt")]
        );
        assert_eq!(fs::metadata(original.join("src/main.py"))?.ino(), inode);
        assert_eq!(
            fs::read_to_string(original.join("src/__pycache__/local.pyc"))?,
            "private cache"
        );
        Ok(())
    }

    #[test]
    fn recovery_restores_original_after_partial_publication_with_missing_manifest() -> Result<()> {
        let (_root, ctx, project) = fixture()?;
        let work = templates::Stage::new(project.parent().unwrap())?;
        let copy = work.0.join("project");
        fs::create_dir(&copy)?;
        fs::write(project.join("Cargo.toml"), "original manifest")?;
        fs::write(copy.join("Cargo.toml"), "new manifest")?;
        fs::write(copy.join("Cargo.lock"), "new lock")?;
        let mutation = Mutation {
            project: project.clone(),
            work: work.0.clone(),
            committed: false,
            changes: vec![
                MutationChange {
                    relative: "Cargo.toml".into(),
                    had_original: true,
                    digest: digest_path(&copy.join("Cargo.toml"))?,
                },
                MutationChange {
                    relative: "Cargo.lock".into(),
                    had_original: false,
                    digest: digest_path(&copy.join("Cargo.lock"))?,
                },
            ],
        };
        save_journal(&ctx, &mutation)?;
        // Crash after saving the original manifest, before replacing it, and
        // after publishing another dependency file. No manifest parser can run.
        fs::rename(project.join("Cargo.toml"), work.0.join("backup-0"))?;
        fs::rename(copy.join("Cargo.lock"), project.join("Cargo.lock"))?;
        let _lock = project_prepare::lock_project(&ctx, &json!({"path":project}), &None)?;
        recover(&ctx, &project)?;
        assert_eq!(
            fs::read_to_string(project.join("Cargo.toml"))?,
            "original manifest"
        );
        assert!(!project.join("Cargo.lock").exists());
        assert!(!journal_path(&ctx, &project)?.exists());
        Ok(())
    }

    #[test]
    fn interrupted_recovery_preserves_concurrent_edits_and_backup() -> Result<()> {
        let (_root, ctx, project) = fixture()?;
        let work = templates::Stage::new(project.parent().unwrap())?;
        let copy = work.0.join("project");
        fs::create_dir(&copy)?;
        fs::write(project.join("manifest"), "original")?;
        fs::write(copy.join("manifest"), "new")?;
        let mutation = Mutation {
            project: project.clone(),
            work: work.0.clone(),
            committed: false,
            changes: vec![MutationChange {
                relative: "manifest".into(),
                had_original: true,
                digest: digest_path(&copy.join("manifest"))?,
            }],
        };
        save_journal(&ctx, &mutation)?;
        fs::rename(project.join("manifest"), work.0.join("backup-0"))?;
        fs::rename(copy.join("manifest"), project.join("manifest"))?;
        fs::write(project.join("manifest"), "user edit after interruption")?;
        assert!(recover(&ctx, &project)
            .unwrap_err()
            .to_string()
            .contains("changed after interrupted"));
        assert_eq!(
            fs::read_to_string(project.join("manifest"))?,
            "user edit after interruption"
        );
        assert_eq!(fs::read_to_string(work.0.join("backup-0"))?, "original");
        assert!(journal_path(&ctx, &project)?.exists());
        Ok(())
    }

    #[test]
    fn committed_recovery_keeps_published_changes_and_later_user_edits() -> Result<()> {
        let (_root, ctx, project) = fixture()?;
        let work = templates::Stage::new(project.parent().unwrap())?;
        fs::create_dir(work.0.join("project"))?;
        fs::write(project.join("manifest"), "later user edit")?;
        fs::write(work.0.join("backup-0"), "original")?;
        let mutation = Mutation {
            project: project.clone(),
            work: work.0.clone(),
            committed: true,
            changes: vec![MutationChange {
                relative: "manifest".into(),
                had_original: true,
                digest: "new content hash".into(),
            }],
        };
        save_journal(&ctx, &mutation)?;
        recover(&ctx, &project)?;
        assert_eq!(
            fs::read_to_string(project.join("manifest"))?,
            "later user edit"
        );
        assert!(!journal_path(&ctx, &project)?.exists());
        assert!(!work.0.exists());
        Ok(())
    }

    #[test]
    fn rust_existing_sqlx_requires_the_selected_database_feature() -> Result<()> {
        let (_root, ctx, project) = fixture()?;
        fs::write(project.join("Cargo.toml"), "[package]\nname='app'\nversion='0.1.0'\n[dependencies]\nsqlx={version='=0.9.0',default-features=false,features=['postgres','runtime-tokio']}\n")?;
        let names = declarations(&ctx, "rust", &project)?;
        assert!(package_present(
            "rust",
            Some(&project),
            &names,
            &json!({"name":"sqlx","features":["postgres"]})
        ));
        assert!(!package_present(
            "rust",
            Some(&project),
            &names,
            &json!({"name":"sqlx","features":["mysql"]})
        ));
        fs::write(project.join("Cargo.toml"), "[package]\nname='app'\nversion='0.1.0'\n[dev-dependencies]\nsqlx={version='=0.9.0',features=['postgres']}\n")?;
        let names = declarations(&ctx, "rust", &project)?;
        assert!(!package_present(
            "rust",
            Some(&project),
            &names,
            &json!({"name":"sqlx","features":["postgres"]})
        ));
        Ok(())
    }

    #[test]
    fn java_managed_and_inactive_profile_dependencies_are_not_installed() -> Result<()> {
        let (_root, ctx, project) = fixture()?;
        fs::write(
            project.join("pom.xml"),
            r#"<project><dependencyManagement><dependencies><dependency><groupId>org.postgresql</groupId><artifactId>postgresql</artifactId></dependency></dependencies></dependencyManagement><profiles><profile><dependencies><dependency><groupId>com.rabbitmq</groupId><artifactId>amqp-client</artifactId></dependency></dependencies></profile></profiles><dependencies><dependency><groupId>redis.clients</groupId><artifactId>jedis</artifactId></dependency></dependencies></project>"#,
        )?;
        let names = declarations(&ctx, "java", &project)?;
        assert!(!names.contains(&normal("org.postgresql:postgresql")));
        assert!(!names.contains(&normal("com.rabbitmq:amqp-client")));
        assert!(names.contains(&normal("redis.clients:jedis")));
        Ok(())
    }
    #[test]
    fn client_additions_cannot_advance_existing_native_pins() -> Result<()> {
        let (root, _ctx, original) = fixture()?;
        let copy = root.0.join("copy");
        fs::create_dir(&copy)?;
        let cases = [
            ("go", "go.mod", "module example.org/app\nrequire example.org/old v1.2.3\n", "module example.org/app\nrequire example.org/old v1.3.0\n"),
            ("dotnet", "packages.lock.json", r#"{"dependencies":{"net10.0":{"Existing":{"resolved":"1.2.3"}}}}"#, r#"{"dependencies":{"net10.0":{"Existing":{"resolved":"1.3.0"}}}}"#),
            ("node", "package-lock.json", r#"{"packages":{"node_modules/existing":{"version":"1.2.3"}}}"#, r#"{"packages":{"node_modules/existing":{"version":"1.3.0"}}}"#),
            ("python", "requirements.lock", "existing==1.2.3\n", "existing==1.3.0\n"),
            ("rust", "Cargo.lock", "[[package]]\nname='existing'\nversion='1.2.3'\nsource='registry+https://example.org'\n", "[[package]]\nname='existing'\nversion='1.3.0'\nsource='registry+https://example.org'\n"),
        ];
        for (stack, file, before, after) in cases {
            fs::write(original.join(file), before)?;
            fs::write(copy.join(file), before)?;
            preserve_locked_versions(stack, &original, &copy)?;
            fs::write(copy.join(file), after)?;
            assert!(
                preserve_locked_versions(stack, &original, &copy)
                    .unwrap_err()
                    .to_string()
                    .contains("original project was left unchanged"),
                "{stack}"
            );
            assert_eq!(fs::read_to_string(original.join(file))?, before);
        }
        Ok(())
    }
}

#[cfg(test)]
mod swift_product_tests {
    use super::*;
    #[test]
    fn declaration_without_app_product_is_not_included() {
        let package = json!({"name":"bunny-swift","products":["BunnySwift"]});
        let mut description = json!({"dependencies":[{"sourceControl":[{"identity":"bunny-swift"}]}],"targets":[{"type":"executable","name":"App","dependencies":[]},{"type":"test","name":"Tests","dependencies":[{"product":["BunnySwift","bunny-swift",null,null]}]}]});
        assert!(!package_present(
            "swift",
            None,
            &swift_target_products(&description),
            &package
        ));
        description["targets"][0]["dependencies"] =
            json!([{"product":["BunnySwift","Bunny-Swift",null,null]}]);
        assert!(package_present(
            "swift",
            None,
            &swift_target_products(&description),
            &package
        ));
        description["dependencies"] = json!([]);
        assert!(!package_present(
            "swift",
            None,
            &swift_target_products(&description),
            &package
        ));
    }
}
