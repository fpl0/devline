//! Bundled client packs are sealed separately from project readiness receipts.
use crate::{safe_catalog_path, text_arg, Context};
use anyhow::{bail, Context as _, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

fn digest(path: &Path) -> Result<String> {
    let mut input = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let size = input.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn files(root: &Path, relative: &Path, result: &mut BTreeMap<PathBuf, u64>) -> Result<()> {
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        let path = relative.join(entry.file_name());
        if path == Path::new("ready.json") || path == Path::new("inventory.json") {
            continue;
        }
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir() {
            files(root, &path, result)?;
        } else if metadata.is_file() {
            result.insert(path, metadata.len());
        } else {
            bail!("Client pack contains a symbolic link or special file");
        }
    }
    Ok(())
}
pub(super) fn check(ctx: &Context, stack: &str, ready: &Value, verify_bytes: bool) -> Result<()> {
    let root = safe_catalog_path(ctx, &format!("service-clients/artifacts/{stack}"))?;
    check_root(&root, stack, ready, verify_bytes)
}
fn check_root(root: &Path, stack: &str, ready: &Value, verify_bytes: bool) -> Result<()> {
    if ready["schema_version"] != 1 || ready["stack"] != stack {
        bail!("Invalid client pack identity");
    }
    let path = root.join("inventory.json");
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
        bail!("Invalid client pack inventory");
    }
    if digest(&path)? != text_arg(ready, "inventory_sha256")? {
        bail!("Client pack inventory checksum failed");
    }
    let inventory: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let mut actual = BTreeMap::new();
    files(root, Path::new(""), &mut actual)?;
    let entries = inventory["files"]
        .as_array()
        .context("Client pack has no artifact inventory")?;
    if actual.len() != entries.len() {
        bail!("Client pack files do not match its sealed inventory");
    }
    for entry in entries {
        let relative = Path::new(text_arg(entry, "path")?);
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            bail!("Invalid client artifact path");
        }
        let size = actual
            .remove(relative)
            .context("Client pack is missing a prepared artifact")?;
        if entry["bytes"].as_u64() != Some(size) {
            bail!("Client artifact size changed: {}", relative.display());
        }
        if verify_bytes && digest(&root.join(relative))? != text_arg(entry, "sha256")? {
            bail!("Client artifact checksum failed: {}", relative.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn sealed_artifacts_reject_missing_changed_extra_and_redirected_files() -> Result<()> {
        let stage = crate::templates::Stage::new(&std::env::temp_dir())?;
        let file = stage.0.join("client.zip");
        fs::write(&file, b"good")?;
        let inventory = json!({"files":[{"path":"client.zip","bytes":4,"sha256":digest(&file)?}]});
        fs::write(
            stage.0.join("inventory.json"),
            serde_json::to_vec(&inventory)?,
        )?;
        let ready = json!({"schema_version":1,"stack":"test","inventory_sha256":digest(&stage.0.join("inventory.json"))?});
        check_root(&stage.0, "test", &ready, true)?;
        fs::write(&file, b"evil")?;
        check_root(&stage.0, "test", &ready, false)?;
        assert!(check_root(&stage.0, "test", &ready, true).is_err());
        fs::remove_file(&file)?;
        assert!(check_root(&stage.0, "test", &ready, false).is_err());
        std::os::unix::fs::symlink(stage.0.join("inventory.json"), &file)?;
        assert!(check_root(&stage.0, "test", &ready, false).is_err());
        fs::remove_file(&file)?;
        fs::write(&file, b"good")?;
        fs::write(stage.0.join("extra"), b"unsealed")?;
        assert!(check_root(&stage.0, "test", &ready, false).is_err());
        Ok(())
    }
}
