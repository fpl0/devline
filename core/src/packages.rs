//! A shelf of popular packages kept on this Mac so a project can add them without a connection.
//!
//! The catalog names the packages. Downloading resolves each one with the runtime's own package
//! manager and stores the native archives in the library. Adding to a project goes through the
//! same staged, network-denied verification as service clients.
use crate::{
    project_prepare, safe_catalog_path, service_clients, tasks::Job, templates, text_arg, Context,
};
use anyhow::{bail, Context as _, Result};
use fs2::FileExt;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// The most packages one request may add, so a typo cannot rewrite a whole manifest.
const ADD_LIMIT: usize = 24;

fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map(Vec::as_slice).unwrap_or_default()
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 200
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"@/._-:[],+".contains(&byte))
}

fn definition(ctx: &Context, stack: &str) -> Result<Value> {
    if !service_clients::STACKS.contains(&stack) {
        bail!("Choose one supported runtime for the package shelf");
    }
    let file = safe_catalog_path(ctx, &format!("packages/{stack}.json"))?;
    let value: Value = serde_json::from_slice(&fs::read(file)?)?;
    if value["schema_version"] != 1 || value["stack"] != stack {
        bail!("Invalid package shelf catalog for {stack}");
    }
    let mut seen = BTreeSet::new();
    for package in rows(&value, "packages") {
        let name = text_arg(package, "name")?;
        if !valid_name(name) || !seen.insert(name) {
            bail!("Invalid package shelf entry: {name}");
        }
    }
    Ok(value)
}

fn shelf_dir(ctx: &Context, stack: &str) -> PathBuf {
    ctx.home.join("packages").join(stack)
}

/// The record of what a completed download put on the shelf.
fn shelf(ctx: &Context, stack: &str) -> Option<Value> {
    let bytes = fs::read(shelf_dir(ctx, stack).join("ready.json")).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    (value["schema_version"] == 1 && value["stack"] == stack).then_some(value)
}

fn summary(definition: &Value, ready: Option<&Value>) -> Value {
    let total =
        rows(definition, "packages").len() + ready.map_or(0, |ready| rows(ready, "custom").len());
    match ready {
        Some(ready) => json!({
            "ready": true,
            "count": rows(ready, "packages").len(),
            "total": total,
            "bytes": ready["bytes"],
            "prepared_at": ready["prepared_at"],
            "toolchain": ready["toolchain"],
            "failed": ready["failed"],
        }),
        None => json!({"ready": false, "count": 0, "total": total, "failed": []}),
    }
}

fn stack_catalog(ctx: &Context, stack: &str, project: Option<&Path>) -> Result<Value> {
    let definition = definition(ctx, stack)?;
    let ready = shelf(ctx, stack);
    let declared = project
        .map(|path| service_clients::declarations(ctx, stack, path))
        .transpose()?
        .unwrap_or_default();
    let custom = ready
        .as_ref()
        .map(|ready| rows(ready, "custom").to_vec())
        .unwrap_or_default();
    let mut packages = Vec::new();
    for entry in rows(&definition, "packages").iter().chain(&custom) {
        let name = text_arg(entry, "name")?;
        let stored = ready.as_ref().and_then(|ready| {
            rows(ready, "packages")
                .iter()
                .find(|package| package["name"] == name)
        });
        let failure = ready.as_ref().and_then(|ready| {
            rows(ready, "failed")
                .iter()
                .find(|package| package["name"] == name)
        });
        let mut row = entry.clone();
        row["available"] = json!(stored.is_some());
        row["version"] = stored.map_or(Value::Null, |package| package["version"].clone());
        row["error"] = failure.map_or(Value::Null, |package| package["error"].clone());
        if let Some(path) = project {
            let candidate = stored.unwrap_or(entry);
            row["installed"] = json!(service_clients::package_present(
                stack,
                Some(path),
                &declared,
                candidate
            ));
        }
        packages.push(row);
    }
    Ok(json!({
        "stack": stack,
        "registry": definition["registry"],
        "shelf": summary(&definition, ready.as_ref()),
        "packages": packages,
    }))
}

/// `packages.catalog` takes `{path?}` for one project's runtime, `{stack?}` for one runtime,
/// or nothing for every runtime with a shelf catalog.
pub(crate) fn catalog(ctx: &Context, request: &Value) -> Result<Value> {
    if request["path"].is_string() {
        let (stack, path) = service_clients::target(ctx, &json!({"path": request["path"]}))?;
        return Ok(json!({"stacks": [stack_catalog(ctx, &stack, path.as_deref())?]}));
    }
    if let Some(stack) = request["stack"].as_str() {
        return Ok(json!({"stacks": [stack_catalog(ctx, stack, None)?]}));
    }
    let mut stacks = Vec::new();
    for stack in service_clients::STACKS {
        if safe_catalog_path(ctx, &format!("packages/{stack}.json")).is_ok() {
            stacks.push(stack_catalog(ctx, stack, None)?);
        }
    }
    Ok(json!({"stacks": stacks}))
}

fn lock(ctx: &Context, stack: &str) -> Result<fs::File> {
    let root = ctx.home.join("packages");
    fs::create_dir_all(&root)?;
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(format!("{stack}.lock")))?;
    file.try_lock_exclusive()
        .ok()
        .with_context(|| format!("The {stack} package shelf is already being downloaded"))?;
    Ok(file)
}

fn clone_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    let mut from = source.as_os_str().to_owned();
    from.push("/.");
    for flags in ["-cR", "-R"] {
        let status = Command::new("/bin/cp")
            .arg(flags)
            .arg(&from)
            .arg(destination)
            .stderr(std::process::Stdio::null())
            .status()?;
        if status.success() {
            return Ok(());
        }
    }
    bail!("Could not copy the existing package shelf")
}

/// `packages.prepare` takes `{id: stack, packages?: [names]}`. It needs a connection: every
/// catalog package, plus any extra names given now or earlier, is resolved again and whatever is
/// new is downloaded. Packages that cannot be resolved are reported, not fatal.
pub(crate) fn prepare(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let stack = text_arg(request, "id")?;
    let definition = definition(ctx, stack)?;
    if request["offline"] == true {
        bail!("Downloading the package shelf needs a connection.");
    }
    let reports = templates::runtime_reports(ctx, &[stack.to_owned()])?;
    let report = reports
        .as_array()
        .and_then(|rows| rows.first())
        .context("No runtime report")?;
    if report["available"] != true {
        bail!(
            "{}",
            report["detail"]
                .as_str()
                .unwrap_or("Install this runtime before downloading its packages")
        );
    }
    let tool = PathBuf::from(text_arg(report, "path")?);
    let toolchain = text_arg(report, "toolchain")?;
    let python = if stack == "python" {
        tool.clone()
    } else {
        templates::python3().context(
            "Downloading packages uses Python 3 for its helper scripts. Install Python, or Apple's Command Line Tools with xcode-select --install",
        )?
    };
    let _lock = lock(ctx, stack)?;
    let previous = shelf(ctx, stack);
    let mut custom: Vec<Value> = previous
        .as_ref()
        .map(|ready| rows(ready, "custom").to_vec())
        .unwrap_or_default();
    // Keeping one more package on an existing shelf fetches just that package.
    let only: Vec<&str> = if previous.is_some() {
        rows(request, "packages")
            .iter()
            .filter_map(Value::as_str)
            .collect()
    } else {
        Vec::new()
    };
    for name in rows(request, "packages") {
        let name = name.as_str().context("Extra packages are given by name")?;
        if !valid_name(name) {
            bail!("Invalid package name: {name}");
        }
        if stack == "swift" {
            bail!("Swift packages need a repository URL and product names; add them to the catalog instead");
        }
        let listed = rows(&definition, "packages")
            .iter()
            .chain(&custom)
            .any(|package| package["name"] == name);
        if !listed {
            custom.push(json!({"name": name, "summary": "Added by you.", "category": "Added"}));
        }
    }
    let root = ctx.home.join("packages");
    let work = templates::Stage::new(&root)?;
    let out = work.0.join("shelf");
    let existing = shelf_dir(ctx, stack);
    if previous.is_some() {
        if let Some(job) = job.as_deref_mut() {
            job.stage("Starting from the packages already on the shelf")?;
        }
        clone_tree(&existing, &out)?;
    } else {
        fs::create_dir(&out)?;
    }
    let home = work.0.join("home");
    let tmp = work.0.join("tmp");
    fs::create_dir(&home)?;
    fs::create_dir(&tmp)?;
    let plan = work.0.join("plan.json");
    let mut wanted = rows(&definition, "packages").to_vec();
    wanted.extend(custom.iter().cloned());
    fs::write(
        &plan,
        serde_json::to_vec(&json!({
            "stack": stack,
            "packages": wanted,
            "custom": custom,
            "only": only,
            "previous": previous,
            "catalog_dir": ctx.catalog_dir.canonicalize()?,
            "toolchain": report["detail"],
        }))?,
    )?;
    let builder = safe_catalog_path(ctx, "packages/build-shelf.py")?;
    let writable = serde_json::to_string(&work.0.canonicalize()?)?;
    let platform = if stack == "dotnet" {
        crate::templates::DOTNET_SANDBOX
    } else {
        ""
    };
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command
        .args([
            "-p",
            &format!("(version 1)(allow default)(deny file-write*)(allow file-write* (subpath {writable}) (literal \"/dev/null\")){platform}"),
        ])
        .arg(&python)
        .arg(&builder)
        .arg(&plan)
        .arg(&out)
        .arg(&tool)
        .current_dir(&work.0)
        .env_clear()
        .env("HOME", &home)
        .env("TMPDIR", &tmp)
        .env("LANG", "en_US.UTF-8");
    templates::project_tool_environment(&mut command, toolchain, &tool)?;
    let progress = work.0.join("progress");
    command.env("DEVLINE_PROGRESS", &progress);
    let label = format!("Downloading {stack} packages");
    match job.as_deref_mut() {
        Some(job) => job.command_reporting(&mut command, &label, Some(&progress))?,
        None => templates::run_command(&mut command, &label, None)?,
    }
    let ready: Value = serde_json::from_slice(
        &fs::read(out.join("ready.json")).context("The download did not finish")?,
    )?;
    if ready["schema_version"] != 1 || ready["stack"] != stack {
        bail!("The downloaded package shelf is incomplete");
    }
    if rows(&ready, "packages").is_empty() {
        let reason = rows(&ready, "failed")
            .first()
            .and_then(|failure| failure["error"].as_str())
            .unwrap_or("Check your connection and try again.");
        bail!("No {stack} packages could be downloaded. {reason}");
    }
    if let Some(job) = job {
        job.stage("Putting the packages on the shelf")?;
    }
    if existing.exists() {
        fs::rename(&existing, work.0.join("previous"))?;
    }
    fs::rename(&out, &existing)?;
    stack_catalog(ctx, stack, None)
}

/// `packages.add` takes `{path, packages: [names], offline?}` and adds shelf packages to the
/// project's manifest and offline dependency cache. With `offline`, nothing is fetched.
pub(crate) fn add(ctx: &Context, request: &Value, job: Option<&mut Job>) -> Result<Value> {
    let names = request["packages"]
        .as_array()
        .context("Choose the packages to add")?;
    if names.is_empty() {
        return Ok(json!({"changed":false,"complete":true,"status":"unchanged","packages":[]}));
    }
    if names.len() > ADD_LIMIT {
        bail!("Add at most {ADD_LIMIT} packages at a time");
    }
    let path = project_prepare::mutation_root(ctx, Path::new(text_arg(request, "path")?))?;
    let _lock = project_prepare::lock_project(ctx, &path, &job)?;
    project_prepare::recover_locked(ctx, &path)?;
    let (stack, root) = service_clients::target(ctx, &json!({"path": path}))?;
    if root.as_deref() != Some(&path) {
        bail!("The project root changed. Select the project again.");
    }
    let ready = shelf(ctx, &stack).with_context(|| {
        format!(
            "Download the {stack} package shelf while you are connected, then add packages offline"
        )
    })?;
    let declared = service_clients::declarations(ctx, &stack, &path)?;
    let mut packages = Vec::new();
    let mut seen = BTreeSet::new();
    for name in names {
        let name = name.as_str().context("Packages are chosen by name")?;
        if !seen.insert(name) {
            continue;
        }
        let package = rows(&ready, "packages")
            .iter()
            .find(|package| package["name"] == name)
            .with_context(|| format!("{name} is not on the {stack} package shelf"))?;
        if !service_clients::package_present(&stack, Some(&path), &declared, package) {
            packages.push(package.clone());
        }
    }
    if packages.is_empty() {
        return Ok(
            json!({"changed":false,"complete":true,"status":"already_included","packages":names}),
        );
    }
    let selected = vec![json!({
        "id": format!("{stack}:shelf"),
        "name": "Package shelf",
        "packages": packages,
        "builtin": false,
    })];
    let mut result = service_clients::publish(
        ctx,
        &path,
        &stack,
        selected,
        Some(&shelf_dir(ctx, &stack)),
        "packages",
        request["offline"] == true,
        json!(names),
        job,
    )?;
    result["packages"] = json!(names);
    Ok(result)
}

/// `packages.remove` takes `{id: stack}` and frees the shelf's disk space.
pub(crate) fn remove(ctx: &Context, request: &Value) -> Result<Value> {
    let stack = text_arg(request, "id")?;
    definition(ctx, stack)?;
    let _lock = lock(ctx, stack)?;
    let existing = shelf_dir(ctx, stack);
    let removed = existing.exists();
    if removed {
        let work = templates::Stage::new(&ctx.home.join("packages"))?;
        fs::rename(&existing, work.0.join("removed"))?;
    }
    Ok(json!({"stack": stack, "removed": removed}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(packages: &str) -> Result<(templates::Stage, Context)> {
        let root = templates::Stage::new(&std::env::temp_dir().canonicalize()?)?;
        let home = root.0.join("home");
        let catalog_dir = root.0.join("catalog");
        fs::create_dir_all(&home)?;
        fs::create_dir_all(catalog_dir.join("packages"))?;
        fs::write(
            catalog_dir.join("packages/python.json"),
            format!(
                r#"{{"schema_version":1,"stack":"python","registry":"PyPI","packages":{packages}}}"#
            ),
        )?;
        let ctx = Context {
            home,
            catalog_dir,
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        Ok((root, ctx))
    }

    const TWO: &str = r#"[{"name":"httpx","summary":"HTTP client.","category":"HTTP"},
                         {"name":"rich","summary":"Terminal output.","category":"CLI"}]"#;

    #[test]
    fn catalog_reports_what_is_on_the_shelf_and_what_failed() -> Result<()> {
        let (_root, ctx) = fixture(TWO)?;
        let empty = catalog(&ctx, &json!({}))?;
        let stack = &empty["stacks"][0];
        assert_eq!(empty["stacks"].as_array().unwrap().len(), 1);
        assert_eq!(stack["shelf"]["ready"], false);
        assert_eq!(stack["shelf"]["total"], 2);
        assert_eq!(stack["packages"][0]["available"], false);

        let shelf = shelf_dir(&ctx, "python");
        fs::create_dir_all(&shelf)?;
        fs::write(
            shelf.join("ready.json"),
            json!({
                "schema_version": 1, "stack": "python", "bytes": 1234, "prepared_at": 5,
                "packages": [{"name": "httpx", "version": "0.28.1"}],
                "failed": [{"name": "rich", "error": "No matching distribution"}],
                "custom": [{"name": "polars", "summary": "Added by you.", "category": "Added"}],
            })
            .to_string(),
        )?;
        let ready = catalog(&ctx, &json!({"stack": "python"}))?;
        let stack = &ready["stacks"][0];
        assert_eq!(stack["shelf"]["ready"], true);
        assert_eq!(stack["shelf"]["count"], 1);
        assert_eq!(stack["shelf"]["total"], 3);
        assert_eq!(stack["packages"][0]["version"], "0.28.1");
        assert_eq!(stack["packages"][0]["available"], true);
        assert_eq!(stack["packages"][1]["available"], false);
        assert_eq!(stack["packages"][1]["error"], "No matching distribution");
        assert_eq!(stack["packages"][2]["name"], "polars");

        // A shelf written for another runtime is never trusted.
        fs::write(
            shelf.join("ready.json"),
            json!({"schema_version": 1, "stack": "go", "packages": []}).to_string(),
        )?;
        assert_eq!(
            catalog(&ctx, &json!({"stack": "python"}))?["stacks"][0]["shelf"]["ready"],
            false
        );
        Ok(())
    }

    #[test]
    fn catalog_entries_must_be_unique_and_safe_to_pass_to_a_package_manager() -> Result<()> {
        for packages in [
            r#"[{"name":"httpx"},{"name":"httpx"}]"#,
            r#"[{"name":"--index-url=https://example.invalid"}]"#,
            r#"[{"name":"rich; rm -rf ~"}]"#,
            r#"[{"name":""}]"#,
        ] {
            let (_root, ctx) = fixture(packages)?;
            assert!(
                catalog(&ctx, &json!({"stack": "python"})).is_err(),
                "{packages}"
            );
        }
        let (_root, ctx) = fixture(TWO)?;
        assert!(catalog(&ctx, &json!({"stack": "cobol"})).is_err());
        Ok(())
    }

    #[test]
    fn downloading_never_starts_offline_and_removing_an_absent_shelf_is_harmless() -> Result<()> {
        let (_root, ctx) = fixture(TWO)?;
        let refused = prepare(&ctx, &json!({"id": "python", "offline": true}), None);
        assert!(refused.unwrap_err().to_string().contains("connection"));
        assert_eq!(remove(&ctx, &json!({"id": "python"}))?["removed"], false);
        fs::create_dir_all(shelf_dir(&ctx, "python"))?;
        assert_eq!(remove(&ctx, &json!({"id": "python"}))?["removed"], true);
        assert!(!shelf_dir(&ctx, "python").exists());
        Ok(())
    }
}
