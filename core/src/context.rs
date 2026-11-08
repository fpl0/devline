//! Shared library location, catalog and SQLite bootstrap.

use crate::doc_index;
use anyhow::{bail, Context as _, Result};
use fs2::FileExt;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{ffi::OsString, fs, path::PathBuf, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocPack {
    pub id: String,
    pub name: String,
    pub stack: String,
    pub version: String,
    pub description: String,
    pub source_url: String,
    pub license: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markdown: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shards: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coverage: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omissions: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub local_only: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub stack: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub bundled_revision: u32,
    pub description: String,
    pub app_type: String,
    pub path: String,
    pub toolchain: String,
    pub minimum_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_macos: Option<String>,
    pub test_command: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
    #[serde(default)]
    pub verified: bool,
    pub target: String,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    pub docs: Vec<DocPack>,
    pub templates: Vec<Template>,
}
#[derive(Clone)]
pub struct Context {
    pub home: PathBuf,
    pub catalog_dir: PathBuf,
    pub catalog: Catalog,
}
impl Context {
    pub fn new(request: &Value) -> Result<Self> {
        let home_override = request
            .get("home")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .or_else(|| branded_env("HOME").map(PathBuf::from));
        let home = match home_override {
            Some(path) => path,
            None => default_home()?,
        };
        let catalog_dir = request
            .get("catalog")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .or_else(|| branded_env("CATALOG").map(PathBuf::from))
            .unwrap_or_else(default_catalog);
        let catalog: Catalog = serde_json::from_slice(
            &fs::read(catalog_dir.join("catalog.json")).with_context(|| {
                format!(
                    "Catalog missing at {}. Set DEVLINE_CATALOG to its directory.",
                    catalog_dir.display()
                )
            })?,
        )?;
        if catalog.schema_version != 1 {
            bail!("Unsupported catalog format {}", catalog.schema_version);
        }
        fs::create_dir_all(&home)?;
        Ok(Self {
            home,
            catalog_dir,
            catalog,
        })
    }
    pub fn database(&self) -> Result<Connection> {
        // First launch requests several panels concurrently. SQLite cannot
        // always wait when two connections both change the journal mode.
        // Serialize only bootstrap/migration; returned readers stay concurrent.
        let bootstrap = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.home.join("database-bootstrap.lock"))?;
        bootstrap.lock_exclusive()?;
        let mut db = Connection::open(self.home.join("library.sqlite3"))?;
        db.busy_timeout(Duration::from_secs(10))?;
        let journal: String = db.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
        if !journal.eq_ignore_ascii_case("wal") {
            db.pragma_update(None, "journal_mode", "WAL")?;
        }
        db.execute_batch("CREATE TABLE IF NOT EXISTS packs(id TEXT, version TEXT, digest TEXT NOT NULL, count INTEGER, PRIMARY KEY(id,version));
            CREATE TABLE IF NOT EXISTS documents(id TEXT PRIMARY KEY, pack_id TEXT, title TEXT, body TEXT, html TEXT, stack TEXT, version TEXT, source_url TEXT, category TEXT);
            CREATE TABLE IF NOT EXISTS aliases(alias TEXT, document_id TEXT);
            CREATE INDEX IF NOT EXISTS aliases_lookup ON aliases(alias);
            CREATE INDEX IF NOT EXISTS documents_pack ON documents(pack_id,version);
            CREATE TABLE IF NOT EXISTS doc_navigation(document_id TEXT PRIMARY KEY, group_name TEXT NOT NULL) WITHOUT ROWID;
            CREATE TABLE IF NOT EXISTS doc_alias_anchors(document_id TEXT NOT NULL, alias TEXT NOT NULL, anchor TEXT NOT NULL, PRIMARY KEY(document_id,alias)) WITHOUT ROWID;
            CREATE TRIGGER IF NOT EXISTS doc_navigation_remove_document AFTER DELETE ON documents BEGIN
                DELETE FROM doc_navigation WHERE document_id=old.id;
                DELETE FROM doc_alias_anchors WHERE document_id=old.id;
            END;
            CREATE VIRTUAL TABLE IF NOT EXISTS docs_fts USING fts5(title,body,stack,content='documents',content_rowid='rowid');")?;
        doc_index::ensure(&mut db)?;
        Ok(db)
    }
}

pub(crate) fn branded_env(name: &str) -> Option<OsString> {
    [format!("DEVLINE_{name}"), format!("DEVKIT_{name}")]
        .into_iter()
        .find_map(|key| std::env::var_os(key).filter(|value| !value.is_empty()))
}

fn default_home() -> Result<PathBuf> {
    let support = PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("Library/Application Support");
    let home = support.join("Devline");
    let previous = support.join("DevKit");
    if !home.try_exists()? && previous.try_exists()? {
        return Ok(previous);
    }
    Ok(home)
}

fn default_catalog() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        for relative in ["../Resources/catalog", "catalog", "../catalog"] {
            let path = exe.parent().unwrap().join(relative);
            if path.join("catalog.json").exists() {
                return path;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data")
}
pub fn lock_library(ctx: &Context) -> Result<fs::File> {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(ctx.home.join("library.lock"))?;
    file.lock_exclusive()?;
    Ok(file)
}
pub fn safe_catalog_path(ctx: &Context, relative: &str) -> Result<PathBuf> {
    let root = ctx.catalog_dir.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(&root) {
        bail!("Catalog path escapes its root");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates;

    #[test]
    fn a_fresh_library_can_open_concurrent_panels() -> Result<()> {
        let stage = templates::Stage::new(&std::env::temp_dir())?;
        let ctx = Context {
            home: stage.0.clone(),
            catalog_dir: stage.0.clone(),
            catalog: Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        let barrier = std::sync::Barrier::new(12);
        std::thread::scope(|scope| -> Result<()> {
            let mut joins = Vec::new();
            for _ in 0..12 {
                joins.push(scope.spawn(|| -> Result<()> {
                    barrier.wait();
                    let db = ctx.database()?;
                    let mode: String =
                        db.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
                    assert_eq!(mode, "wal");
                    assert_eq!(
                        db.query_row("SELECT count(*) FROM packs", [], |row| row.get::<_, u64>(0))?,
                        0
                    );
                    Ok(())
                }));
            }
            for join in joins {
                join.join().expect("database thread")?;
            }
            Ok(())
        })
    }
}
