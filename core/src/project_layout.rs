//! Project layout at creation time. Starter releases and legacy projects stay intact.
mod assets;

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{fs, io, os::unix::fs::PermissionsExt, path::Path};

const DIRECTORY: &str = ".devline";
pub(crate) const ORIGIN: &str = ".devline/origin.json";
pub(crate) const LEGACY_ORIGIN: &str = ".devkit-origin.json";
pub(crate) const CONFIG: &str = ".devline/config.json";
const SCRIPTS: &str = ".devline/scripts";
pub(crate) const CACHE: &str = ".devline/cache";
pub(crate) const LEGACY_CACHE: &str = ".devline-offline";
pub(crate) const PACKAGES: &str = ".devline/packages";
pub(crate) const SERVICES: &str = ".devline/services";
pub(crate) const LEGACY_SERVICES: &str = ".devline-infra";
pub(crate) const TEST: &str = ".devline/scripts/test-offline.sh";
pub(crate) const LEGACY_TEST: &str = "test-offline.sh";
pub(crate) const TEST_COMMAND: &str = "./devline.sh test";

pub(crate) fn is_organized(root: &Path) -> bool {
    fs::symlink_metadata(root.join(DIRECTORY)).is_ok_and(|m| m.is_dir())
        && fs::symlink_metadata(root.join(ORIGIN)).is_ok_and(|m| m.is_file())
}

pub(crate) fn cache_relative(root: &Path) -> &'static str {
    if is_organized(root) {
        CACHE
    } else {
        LEGACY_CACHE
    }
}

pub(crate) fn test_relative(root: &Path) -> &'static str {
    if is_organized(root) && fs::symlink_metadata(root.join(TEST)).is_ok_and(|m| m.is_file()) {
        TEST
    } else {
        LEGACY_TEST
    }
}

pub(crate) fn is_services(relative: &Path) -> bool {
    relative.starts_with(SERVICES) || relative.starts_with(LEGACY_SERVICES)
}

pub(super) fn organize(
    root: &Path,
    stack: &str,
    origin: &Value,
    tool_environment: &str,
    relocate_assets: bool,
) -> Result<()> {
    fs::create_dir(root.join(DIRECTORY))
        .context("Reserve .devline for generated project tooling")?;
    fs::create_dir(root.join(SCRIPTS))?;
    fs::write(root.join(ORIGIN), serde_json::to_vec_pretty(origin)?)?;
    fs::write(root.join(".devline/.gitignore"), "/cache/\n")?;
    if relocate_assets {
        assets::organize(root, stack)?;
    }
    if !root.join(test_relative(root)).is_file() {
        anyhow::bail!("An offline verification script is required");
    }
    write_config(root, stack)?;
    fs::write(root.join(SCRIPTS).join("toolchain.sh"), tool_environment)?;
    fs::write(
        root.join(SCRIPTS).join("lifecycle.sh"),
        include_bytes!("project_layout/lifecycle.sh"),
    )?;
    let launcher = root.join("devline.sh");
    use std::io::Write as _;
    fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&launcher)
        .context("Reserve devline.sh for the project launcher")?
        .write_all(include_bytes!("project_layout/devline.sh"))?;
    fs::set_permissions(launcher, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn write_config(root: &Path, stack: &str) -> Result<()> {
    let legacy = root.join("devline.json");
    let mut config: Value = match fs::read(&legacy) {
        Ok(bytes) => serde_json::from_slice(&bytes).context("Read starter launch configuration")?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let mut config = json!({"schema_version": 1});
            if root.join(SCRIPTS).join("run.sh").is_file() || root.join("run.sh").is_file() {
                config["run"] = json!({"command": ["/bin/sh", "./run.sh"]});
            } else if stack == "dotnet" {
                config["run"] = json!({"command": ["dotnet", "run", "--no-launch-profile"]});
            }
            config
        }
        Err(error) => return Err(error.into()),
    };
    if config["schema_version"] != 1 {
        anyhow::bail!("Unsupported starter launch configuration schema_version");
    }
    if let Some(arguments) = config
        .get_mut("run")
        .and_then(|run| run.get_mut("command"))
        .and_then(Value::as_array_mut)
    {
        for argument in arguments {
            if argument == "./run.sh" && root.join(SCRIPTS).join("run.sh").is_file() {
                *argument = json!("./.devline/scripts/run.sh");
            }
        }
    }
    let test = &config["test"]["command"];
    if test.is_null() || test == &json!(["./test-offline.sh"]) {
        config["test"] = json!({"command": ["./devline.sh", "test"]});
    }
    fs::write(root.join(CONFIG), serde_json::to_vec_pretty(&config)?)?;
    if legacy.exists() {
        fs::remove_file(legacy)?;
    }
    Ok(())
}
