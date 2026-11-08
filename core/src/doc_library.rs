//! Documentation pack selection, source verification and SQLite publication.

use crate::{
    apple_docs, doc_index, doc_updates, lock_library, safe_catalog_path, tasks, text_arg, updates,
    Context, DocPack,
};
use anyhow::{bail, Context as _, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub(crate) struct Document {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) body: String,
    #[serde(default)]
    pub(crate) html: Option<String>,
    pub(crate) source_url: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    category: Option<String>,
}

impl Document {
    pub(crate) fn category(&self, pack: &DocPack) -> &str {
        self.category
            .as_deref()
            .filter(|value| doc_index::valid_category(value))
            .unwrap_or_else(|| doc_index::category(&pack.id, &self.id, &self.source_url))
    }
}

pub(crate) fn valid_doc_id(id: &str) -> bool {
    id.len() <= 64
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

pub(crate) fn is_devdocs(id: &str) -> bool {
    id.strip_prefix("devdocs-")
        .is_some_and(|slug| !slug.is_empty())
}

pub(crate) fn import_local(
    ctx: &Context,
    id: &str,
    version: &str,
    keep_latest: bool,
) -> Result<Value> {
    validate_doc_identity(id, version)?;
    let pack = ctx
        .catalog
        .docs
        .iter()
        .find(|pack| pack.id == id && pack.version == version)
        .with_context(|| {
            format!("Documentation {id}@{version} is unavailable in the import catalog")
        })?;
    if !is_devdocs(id) || !pack.local_only {
        bail!("The local importer requires a local-only DevDocs pack");
    }
    let source = safe_catalog_path(ctx, &pack.path)?;
    let _lock = lock_library(ctx)?;
    let automatic_retention = updates::keep_latest(ctx)?;
    let db = ctx.database()?;
    let active = doc_updates::preferences(&db)?;
    if keep_latest || automatic_retention {
        let mut query = db.prepare("SELECT version FROM packs WHERE id=?1")?;
        let installed = query
            .query_map([id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if let Some(newest) = installed
            .into_iter()
            .max_by(|a, b| updates::compare_doc_releases(a, b))
            .filter(|newest| updates::compare_devdocs_versions(newest, version).is_gt())
        {
            drop(query);
            drop(db);
            let newest = active
                .get(id)
                .filter(|active| updates::compare_devdocs_versions(active, &newest).is_eq())
                .unwrap_or(&newest);
            doc_updates::prune(ctx, Some(&[(id, newest)]))?;
            return Ok(json!({"id":id,"version":newest,"skipped":true}));
        }
    }
    drop(db);
    let prefer = active
        .get(id)
        .is_none_or(|current| !updates::compare_devdocs_versions(version, current).is_lt());
    let cached = doc_updates::store_original(ctx, pack, &source)?;
    let installed = install_doc_pack_with_preference(ctx, pack, &cached, prefer)?;
    if keep_latest && !automatic_retention {
        doc_updates::prune(ctx, Some(&[(id, version)]))?;
    }
    Ok(installed)
}

pub(crate) fn validate_doc_identity(id: &str, version: &str) -> Result<()> {
    if !valid_doc_id(id) || !valid_doc_version(version) {
        bail!("Invalid documentation release identity");
    }
    Ok(())
}

pub(crate) fn valid_doc_version(version: &str) -> bool {
    version.len() <= 128
        && version
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".+-_".contains(&byte))
}

pub(crate) fn install_docs(ctx: &Context, id: &str, version: Option<&str>) -> Result<Value> {
    install_docs_job(ctx, id, version, None)
}

pub(crate) fn install_docs_job(
    ctx: &Context,
    id: &str,
    version: Option<&str>,
    job: Option<&mut tasks::Job>,
) -> Result<Value> {
    if !valid_doc_id(id) {
        bail!("Invalid documentation pack ID");
    }
    if let Some(version) = version {
        validate_doc_identity(id, version)?;
    }
    if id == apple_docs::PACK_ID {
        return apple_docs::install(ctx, version, job);
    }
    let _lock = lock_library(ctx)?;
    let pack = ctx
        .catalog
        .docs
        .iter()
        .filter(|p| p.id == id && version.is_none_or(|v| v == p.version))
        .max_by(|a, b| updates::compare_doc_releases(&a.version, &b.version));
    // Do not verify or select an equal/older cache when original files are bundled.
    if let Some(bundled) = pack {
        if version.is_some() {
            return install_doc_pack(ctx, bundled, &safe_catalog_path(ctx, &bundled.path)?);
        }
        let newer = doc_updates::cached_packs(ctx)?
            .into_iter()
            .filter(|(candidate, _)| {
                candidate.id == id
                    && updates::compare_doc_releases(&candidate.version, &bundled.version).is_gt()
            })
            .max_by(|(a, _), (b, _)| updates::compare_doc_releases(&a.version, &b.version));
        if let Some((candidate, _)) = newer {
            if let Some((cached, path)) =
                doc_updates::cached_pack(ctx, id, Some(&candidate.version))?
            {
                return install_doc_pack(ctx, &cached, &path);
            }
        }
    } else if let Some((cached, path)) = doc_updates::cached_pack(ctx, id, version)? {
        return install_doc_pack(ctx, &cached, &path);
    }
    let Some(pack) = pack else {
        if let Some(version) = version {
            let db = ctx.database()?;
            let count: Option<u64> = db
                .query_row(
                    "SELECT count FROM packs WHERE id=?1 AND version=?2",
                    params![id, version],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(count) = count {
                return Ok(json!({"id":id,"version":version,"count":count}));
            }
        }
        bail!("Documentation pack {id} is unavailable in the catalog");
    };
    let directory = safe_catalog_path(ctx, &pack.path)?;
    install_doc_pack(ctx, pack, &directory)
}

const DOC_SHARD_LIMIT: u64 = 128 * 1024 * 1024;

pub(crate) fn shard_name(shard: usize) -> String {
    if shard == 1 {
        "documents.json".to_owned()
    } else {
        format!("documents-{shard}.json")
    }
}

pub(crate) fn add_shard_to_pack_digest(hash: &mut Sha256, pack: &DocPack, bytes: &[u8]) {
    // Older single-shard receipts hash the raw file. Frame each shard in a
    // multi-shard release so the boundaries are part of its digest.
    if pack.shards.unwrap_or(1) > 1 {
        hash.update((bytes.len() as u64).to_le_bytes());
    }
    hash.update(bytes);
}

fn open_doc_shard(directory: &Path, pack: &DocPack, shard: usize) -> Result<fs::File> {
    let name = shard_name(shard);
    let path = directory.join(&name);
    let detail = || format!("Read {name} for {}@{}", pack.id, pack.version);
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(not(unix))]
    if !fs::symlink_metadata(&path)?.is_file() {
        bail!("Invalid documentation shard");
    }
    let file = options.open(&path).with_context(detail)?;
    let metadata = file.metadata().with_context(detail)?;
    if !metadata.is_file() || metadata.len() > DOC_SHARD_LIMIT {
        bail!(
            "Invalid documentation shard {name} for {}@{}",
            pack.id,
            pack.version
        );
    }
    Ok(file)
}

pub(crate) fn read_doc_shard(directory: &Path, pack: &DocPack, shard: usize) -> Result<Vec<u8>> {
    let file = open_doc_shard(directory, pack, shard)?;
    let mut bytes = Vec::with_capacity(file.metadata()?.len().min(DOC_SHARD_LIMIT) as usize);
    file.take(DOC_SHARD_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > DOC_SHARD_LIMIT {
        bail!("Documentation shard exceeds its size limit");
    }
    Ok(bytes)
}

pub(crate) fn hash_doc_shard(directory: &Path, pack: &DocPack, shard: usize) -> Result<String> {
    let mut hash = Sha256::new();
    let count = io::copy(
        &mut open_doc_shard(directory, pack, shard)?.take(DOC_SHARD_LIMIT + 1),
        &mut hash,
    )?;
    if count > DOC_SHARD_LIMIT {
        bail!("Documentation shard exceeds its size limit");
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn verify_doc_source<'a>(
    ctx: &'a Context,
    pack: &'a DocPack,
    directory: &Path,
) -> Result<&'a DocPack> {
    validate_doc_identity(&pack.id, &pack.version)?;
    let Some(bundled) = ctx
        .catalog
        .docs
        .iter()
        .find(|known| known.id == pack.id && known.version == pack.version)
    else {
        return Ok(pack);
    };
    let original = safe_catalog_path(ctx, &bundled.path)?;
    if directory.canonicalize()? == original {
        return Ok(bundled);
    }
    let shards = bundled.shards.unwrap_or(1);
    if !(1..=1024).contains(&shards) || pack.shards.unwrap_or(1) != shards {
        bail!(
            "Documentation {}@{} conflicts with the bundled release",
            pack.id,
            pack.version
        );
    }
    for shard in 1..=shards {
        let original = hash_doc_shard(&original, bundled, shard)?;
        let incoming = hash_doc_shard(directory, pack, shard)?;
        if original != incoming {
            bail!(
                "Documentation {}@{} conflicts with the bundled release",
                pack.id,
                pack.version
            );
        }
    }
    Ok(bundled)
}

// Callers hold the library lock from source selection through publication.
pub(crate) fn install_doc_pack(ctx: &Context, pack: &DocPack, directory: &Path) -> Result<Value> {
    install_doc_pack_with_preference(ctx, pack, directory, pack.local_only)
}

pub(crate) fn install_doc_pack_with_preference(
    ctx: &Context,
    pack: &DocPack,
    directory: &Path,
    prefer: bool,
) -> Result<Value> {
    let pack = verify_doc_source(ctx, pack, directory)?;
    let mut db = ctx.database()?;
    let tx = db.transaction()?;
    let mut result = import_doc_pack(&tx, pack, directory)?;
    if prefer {
        doc_updates::prefer_release(&tx, &pack.id, &pack.version)?;
    }
    if pack.local_only {
        result["local_only"] = json!(true);
    }
    tx.commit()?;
    doc_updates::prune_after_install(ctx, &[(pack.id.as_str(), pack.version.as_str())])?;
    Ok(result)
}

// The caller holds the library lock while staging companion starter/cache files.
pub(crate) fn install_doc_packs(ctx: &Context, packs: &[(DocPack, PathBuf)]) -> Result<Vec<Value>> {
    let packs = packs
        .iter()
        .map(|(pack, directory)| Ok((verify_doc_source(ctx, pack, directory)?, directory)))
        .collect::<Result<Vec<_>>>()?;
    let mut db = ctx.database()?;
    let tx = db.transaction()?;
    let results = packs
        .into_iter()
        .map(|(pack, path)| import_doc_pack(&tx, pack, path))
        .collect::<Result<Vec<_>>>()?;
    tx.commit()?;
    Ok(results)
}

fn import_doc_pack(
    tx: &rusqlite::Transaction<'_>,
    pack: &DocPack,
    directory: &Path,
) -> Result<Value> {
    let shards = pack.shards.unwrap_or(1);
    if !(1..=1024).contains(&shards) {
        bail!("Invalid documentation pack shard count");
    }
    let previous: Option<(String, usize)> = tx
        .query_row(
            "SELECT digest,count FROM packs WHERE id=?1 AND version=?2",
            params![pack.id, pack.version],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let mut hash = Sha256::new();
    let mut count = 0;
    {
        let mut document = tx.prepare("INSERT INTO documents(id,pack_id,title,body,html,stack,version,source_url,category) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)")?;
        let mut fts =
            tx.prepare("INSERT INTO docs_fts(rowid,title,body,stack) VALUES(?1,?2,?3,?4)")?;
        let mut aliases = tx.prepare("INSERT INTO aliases VALUES(?1,?2)")?;
        for shard in 1..=shards {
            let bytes = read_doc_shard(directory, pack, shard)?;
            add_shard_to_pack_digest(&mut hash, pack, &bytes);
            if previous.is_some() {
                continue;
            }
            let docs: Vec<Document> = serde_json::from_slice(&bytes)
                .with_context(|| format!("Parse shard {shard} for {}@{}", pack.id, pack.version))?;
            drop(bytes);
            if docs.is_empty() {
                bail!("Documentation pack shard {shard} is empty");
            }
            count += docs.len();
            for doc in docs {
                if doc.id.is_empty() || doc.title.is_empty() {
                    bail!("Document id and title must be present");
                }
                let id = format!("{}@{}/{}", pack.id, pack.version, doc.id);
                let category = doc.category(pack);
                document.execute(params![
                    id,
                    pack.id,
                    doc.title,
                    doc.body,
                    doc.html,
                    pack.stack,
                    pack.version,
                    doc.source_url,
                    category
                ])?;
                fts.execute(params![
                    tx.last_insert_rowid(),
                    doc.title,
                    doc.body,
                    pack.stack
                ])?;
                for alias in std::iter::once(&doc.title).chain(doc.aliases.iter()) {
                    aliases.execute(params![alias.to_lowercase(), id])?;
                    doc_index::add_alias(tx, &id, alias)?;
                }
            }
        }
    }
    let digest = format!("{:x}", hash.finalize());
    if let Some((old, count)) = previous {
        if old != digest {
            bail!(
                "The content of {}@{} changed. Publish a new pack version.",
                pack.id,
                pack.version
            );
        }
        doc_updates::record_installed(tx, pack)?;
        return Ok(json!({"id":pack.id,"version":pack.version,"count":count}));
    }
    tx.execute(
        "INSERT INTO packs VALUES(?1,?2,?3,?4)",
        params![pack.id, pack.version, digest, count],
    )?;
    doc_index::mark_pack(tx, &pack.id, &pack.version)?;
    doc_index::index_pack_sections(tx, pack)?;
    doc_updates::record_installed(tx, pack)?;
    Ok(json!({"id":pack.id,"version":pack.version,"count":count}))
}

pub(crate) fn catalog(ctx: &Context, db: &Connection) -> Result<Vec<Value>> {
    let mut stmt = db.prepare("SELECT id,version FROM packs ORDER BY id,version")?;
    let installed = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<std::collections::HashSet<_>>>()?;
    let mut entries = ctx.catalog.docs.clone();
    for (pack, _) in doc_updates::cached_packs(ctx)? {
        if !entries
            .iter()
            .any(|entry| entry.id == pack.id && entry.version == pack.version)
        {
            entries.push(pack);
        }
    }
    let metadata = doc_updates::installed_metadata(db)?;
    for pack in metadata.values() {
        if !entries
            .iter()
            .any(|entry| entry.id == pack.id && entry.version == pack.version)
        {
            entries.push(pack.clone());
        }
    }
    let preferred = doc_updates::preferences(db)?;
    for (id, version) in &installed {
        if entries
            .iter()
            .any(|pack| pack.id == *id && pack.version == *version)
        {
            continue;
        }
        if let Some(template) = ctx
            .catalog
            .docs
            .iter()
            .filter(|pack| pack.id == *id)
            .max_by(|a, b| updates::compare_doc_releases(&a.version, &b.version))
        {
            let mut older = template.clone();
            older.version.clone_from(version);
            older.markdown = None;
            older.shards = None;
            older.upstream_version = None;
            older.snapshot_date = None;
            older.coverage.clear();
            older.omissions =
                vec!["Coverage metadata was not recorded for this older snapshot.".into()];
            entries.push(older);
        }
    }
    entries.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then(updates::compare_doc_releases(&b.version, &a.version))
    });
    entries
        .into_iter()
        .map(|pack| {
            let present = installed.contains(&(pack.id.clone(), pack.version.clone()));
            let active = preferred.get(&pack.id).map_or_else(
                || {
                    installed
                        .iter()
                        .filter(|(id, _)| id == &pack.id)
                        .max_by(|(_, a), (_, b)| updates::compare_doc_releases(a, b))
                        .is_some_and(|(_, version)| version == &pack.version)
                },
                |version| version == &pack.version,
            );
            let mut value = serde_json::to_value(pack)?;
            value["installed"] = json!(present);
            value["active"] = json!(present && active);
            Ok(value)
        })
        .collect()
}

pub(crate) fn remove(ctx: &Context, request: &Value) -> Result<Value> {
    let _lock = lock_library(ctx)?;
    let id = text_arg(request, "id")?;
    let version = text_arg(request, "version")?;
    let mut db = ctx.database()?;
    let tx = db.transaction()?;
    doc_updates::remove_index(&tx, id, version)?;
    tx.commit()?;
    Ok(json!({"removed":true}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{templates, Catalog};

    #[test]
    fn a_cached_exact_version_cannot_replace_the_bundled_documentation() -> Result<()> {
        let staging = templates::Stage::new(&std::env::temp_dir())?;
        let pack: DocPack = serde_json::from_value(json!({
            "id":"go","version":"1.2.3","path":"docs/go/1.2.3","name":"Go",
            "stack":"go","description":"Pack","source_url":"https://go.dev/","license":"BSD-3-Clause"
        }))?;
        let catalog_dir = staging.0.join("catalog");
        let bundled = catalog_dir.join(&pack.path);
        let conflicting = staging.0.join("conflicting");
        let home = staging.0.join("home");
        fs::create_dir(&home)?;
        for (directory, body) in [
            (&bundled, "Bundled original"),
            (&conflicting, "Conflicting data"),
        ] {
            fs::create_dir_all(directory)?;
            fs::write(
                directory.join("documents.json"),
                serde_json::to_vec(&json!([{
                    "id":"article","title":"Article","body":body,"source_url":"https://go.dev/reference"
                }]))?,
            )?;
        }
        let ctx = Context {
            home,
            catalog_dir,
            catalog: Catalog {
                schema_version: 1,
                docs: vec![pack.clone()],
                templates: Vec::new(),
            },
        };
        assert!(install_doc_pack(&ctx, &pack, &conflicting)
            .unwrap_err()
            .to_string()
            .contains("bundled release"));
        {
            let _lock = lock_library(&ctx)?;
            assert!(install_doc_packs(&ctx, &[(pack.clone(), conflicting.clone())]).is_err());
        }
        assert_eq!(
            ctx.database()?
                .query_row("SELECT count(*) FROM packs", [], |row| row.get::<_, u64>(0))?,
            0
        );
        doc_updates::store_original(&ctx, &pack, &conflicting)?;
        assert_eq!(install_docs(&ctx, "go", None)?["count"], 1);
        assert_eq!(install_docs(&ctx, "go", Some("1.2.3"))?["count"], 1);
        assert_eq!(
            ctx.database()?
                .query_row("SELECT body FROM documents", [], |row| row
                    .get::<_, String>(0))?,
            "Bundled original"
        );
        let mut invalid = pack;
        invalid.version = "../../outside".into();
        assert!(install_doc_pack(&ctx, &invalid, &bundled)
            .unwrap_err()
            .to_string()
            .contains("identity"));
        Ok(())
    }
}
