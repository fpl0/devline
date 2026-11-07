use anyhow::{bail, Context as ErrorContext, Result};
use fs2::FileExt;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    ffi::{CStr, CString},
    fs,
    io::{self, Read},
    os::raw::c_char,
    path::{Path, PathBuf},
    time::Duration,
};
mod apple_docs;
mod content_updates;
mod doc_index;
mod doc_links;
mod doc_search;
mod doc_updates;
mod guides;
mod infrastructure;
mod markdown;
mod offline;
mod project;
mod project_prepare;
pub mod project_run;
mod reference;
mod service_clients;
pub mod tasks;
pub mod templates;
mod toolbox;
mod updates;

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
#[derive(Deserialize)]
struct Document {
    id: String,
    title: String,
    body: String,
    #[serde(default)]
    html: Option<String>,
    source_url: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    category: Option<String>,
}
impl Context {
    pub fn new(request: &Value) -> Result<Self> {
        let home = request
            .get("home")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("DEVKIT_HOME").map(PathBuf::from))
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("Library/Application Support/DevKit")
            });
        let catalog_dir = request
            .get("catalog")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("DEVKIT_CATALOG").map(PathBuf::from))
            .unwrap_or_else(default_catalog);
        let catalog: Catalog = serde_json::from_slice(
            &fs::read(catalog_dir.join("catalog.json")).with_context(|| {
                format!(
                    "Catalog missing at {}. Set DEVKIT_CATALOG to its directory.",
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
            CREATE VIRTUAL TABLE IF NOT EXISTS docs_fts USING fts5(title,body,stack,content='documents',content_rowid='rowid');")?;
        doc_index::ensure(&mut db)?;
        Ok(db)
    }
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
fn text_arg<'a>(r: &'a Value, name: &str) -> Result<&'a str> {
    r.get(name)
        .and_then(Value::as_str)
        .with_context(|| format!("Missing {name}"))
}

fn valid_doc_id(id: &str) -> bool {
    id.len() <= 64
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

pub(crate) fn validate_doc_identity(id: &str, version: &str) -> Result<()> {
    if !valid_doc_id(id)
        || version.is_empty()
        || version.len() > 128
        || !version.as_bytes()[0].is_ascii_alphanumeric()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'-' | b'_'))
    {
        bail!("Invalid documentation release identity");
    }
    Ok(())
}

fn install_docs(ctx: &Context, id: &str, version: Option<&str>) -> Result<Value> {
    install_docs_job(ctx, id, version, None)
}

fn install_docs_job(
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

fn open_doc_shard(directory: &Path, pack: &DocPack, shard: usize) -> Result<fs::File> {
    let name = if shard == 1 {
        "documents.json".to_owned()
    } else {
        format!("documents-{shard}.json")
    };
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

fn read_doc_shard(directory: &Path, pack: &DocPack, shard: usize) -> Result<Vec<u8>> {
    let file = open_doc_shard(directory, pack, shard)?;
    let mut bytes = Vec::with_capacity(file.metadata()?.len().min(DOC_SHARD_LIMIT) as usize);
    file.take(DOC_SHARD_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > DOC_SHARD_LIMIT {
        bail!("Documentation shard exceeds its size limit");
    }
    Ok(bytes)
}

fn hash_doc_shard(directory: &Path, pack: &DocPack, shard: usize) -> Result<String> {
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

fn install_doc_pack_with_preference(
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
            if shards > 1 {
                hash.update((bytes.len() as u64).to_le_bytes());
            }
            hash.update(&bytes);
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
                let category = doc
                    .category
                    .as_deref()
                    .filter(|value| doc_index::valid_category(value))
                    .unwrap_or_else(|| doc_index::category(&pack.id, &doc.id, &doc.source_url));
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
    doc_updates::record_installed(tx, pack)?;
    Ok(json!({"id":pack.id,"version":pack.version,"count":count}))
}

fn catalog_docs(ctx: &Context, db: &Connection) -> Result<Vec<Value>> {
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
pub fn project_context(path: &Path) -> Value {
    project::detected_context(path)
}

pub fn execute(request: &Value) -> Result<Value> {
    let ctx = Context::new(request)?;
    match text_arg(request, "command")? {
        "infra.clients" => service_clients::catalog(&ctx, request),
        "service_clients.add" => service_clients::add(&ctx, request, None),
        command if command.starts_with("infra.") => infrastructure::execute(&ctx, request, None),
        "toolbox.status" => toolbox::status(&ctx, request, false),
        "toolbox.check" => toolbox::status(&ctx, request, true),
        "toolbox.prepare" => toolbox::prepare(&ctx, request, None),
        "project.context" => project::context(&ctx, Path::new(text_arg(request, "path")?)),
        "project.readiness" => project_prepare::readiness(&ctx, request),
        "project.prepare" => project_prepare::prepare(&ctx, request, None),
        "project.runner" => project_prepare::runner(&ctx, request),
        "project.commands" => project_run::commands(&ctx, request),
        "project.start" => project_run::start(&ctx, request),
        "project.stop" => project_run::stop(&ctx, request),
        "project.status" => project_run::status(&ctx, request),
        "project.logs" => project_run::logs(&ctx, request),
        "new.ready" => project_prepare::create(&ctx, request),
        "project.pin" => project::pin(
            &ctx,
            Path::new(text_arg(request, "path")?),
            text_arg(request, "id")?,
            text_arg(request, "version")?,
        ),
        "project.unpin" => project::unpin(
            &ctx,
            Path::new(text_arg(request, "path")?),
            text_arg(request, "id")?,
        ),
        "docs.distribution.status" => doc_updates::status(&ctx),
        "docs.local.status" => {
            if request["id"]
                .as_str()
                .is_some_and(|id| id != apple_docs::PACK_ID)
            {
                bail!("Unknown local documentation family");
            }
            apple_docs::status(&ctx)
        }
        "docs.distribution.check" => doc_updates::check(&ctx),
        "docs.distribution.install" => doc_updates::install(&ctx, request),
        "docs.distribution.rollback" => doc_updates::rollback(&ctx, request),
        "docs.distribution.configure" => doc_updates::configure(&ctx, text_arg(request, "url")?),
        "jobs.start" => tasks::start(&ctx, request),
        "jobs.status" => tasks::status(&ctx, request),
        "jobs.cancel" => tasks::cancel(&ctx, request),
        "jobs.list" => tasks::list(&ctx),
        "offline.status" => offline::status(&ctx, request),
        "offline.check" => templates::verify_offline(&ctx, text_arg(request, "id")?, None),
        "bundle.export" => offline::export(&ctx, request),
        "bundle.inspect" => offline::inspect(&ctx, request),
        "bundle.import" => offline::import(&ctx, request),
        "templates.inspect" => templates::inspect(&ctx, text_arg(request, "id")?),
        "templates.remove" => templates::remove(&ctx, text_arg(request, "id")?),
        "catalog" => {
            let db = ctx.database()?;
            let docs = catalog_docs(&ctx, &db)?;
            Ok(json!({"docs":docs,"templates":templates::list(&ctx)?,"home":ctx.home}))
        }
        "docs.install" => install_docs(&ctx, text_arg(request, "id")?, request["version"].as_str()),
        "docs.search" => doc_search::search(&ctx, request),
        "docs.show" => doc_search::show(&ctx, request),
        "docs.resolve" => doc_links::resolve(&ctx, request),
        "guide.list" => guides::list(&ctx),
        "guide.show" => guides::show(&ctx, text_arg(request, "id")?),
        "reading.list" => reference::list(&ctx),
        "reading.visit" => reference::visit(
            &ctx,
            text_arg(request, "id")?,
            request["anchor"].as_str(),
            request["focus_text"].as_str(),
        ),
        "reading.save" => reference::save(
            &ctx,
            text_arg(request, "id")?,
            request["anchor"].as_str(),
            request["focus_text"].as_str(),
        ),
        "reading.remove" => reference::remove(&ctx, text_arg(request, "id")?),
        "library.retention" => updates::retention(&ctx, request),
        "library.purge" => {
            let _lock = lock_library(&ctx)?;
            let docs = doc_updates::prune(&ctx, None)?;
            let starters = templates::prune(&ctx, None)?;
            drop(_lock);
            let images = infrastructure::prune(&ctx)?;
            Ok(json!({"docs":docs,"starters":starters,"images":images}))
        }
        "docs.remove" => {
            let _lock = lock_library(&ctx)?;
            let id = text_arg(request, "id")?;
            let version = text_arg(request, "version")?;
            let mut db = ctx.database()?;
            let tx = db.transaction()?;
            doc_updates::remove_index(&tx, id, version)?;
            tx.commit()?;
            Ok(json!({"removed":true}))
        }
        "templates.list" => templates::list(&ctx),
        "runtime.list" => templates::runtimes(&ctx),
        "updates.app.status" => updates::app_status(&ctx),
        "updates.app.check" => updates::app_check(&ctx, request),
        "updates.status" => content_updates::status(&ctx),
        "updates.runtime.status" => updates::runtime_status(&ctx),
        "updates.check" => content_updates::check(&ctx, request, None),
        "updates.apply" => content_updates::apply(&ctx, request, None),
        "updates.configure" => updates::configure(&ctx, text_arg(request, "url")?),
        "templates.install" => templates::install(&ctx, text_arg(request, "id")?),
        "doctor" => templates::doctor(&ctx, text_arg(request, "id")?),
        "new" => templates::create(
            &ctx,
            text_arg(request, "id")?,
            Path::new(text_arg(request, "destination")?),
            request["dry_run"].as_bool().unwrap_or(false),
        ),
        "templates.build" => templates::build_latest(&ctx, text_arg(request, "id")?),
        "prepare" => templates::prepare(&ctx, text_arg(request, "id")?, None),
        other => bail!("Unknown command {other}"),
    }
}
/// Execute a JSON request and return an owned, NUL-terminated JSON response.
///
/// # Safety
/// A non-null `input` must point to a readable NUL-terminated string for the
/// duration of this call. Release the returned pointer exactly once with
/// `devkit_free`; do not deallocate it with a foreign allocator.
#[no_mangle]
pub unsafe extern "C" fn devkit_request(input: *const c_char) -> *mut c_char {
    let response = std::panic::catch_unwind(|| -> Result<Value> {
        if input.is_null() {
            bail!("Empty request");
        }
        let request: Value = serde_json::from_str(CStr::from_ptr(input).to_str()?)?;
        execute(&request)
    });
    let value = match response {
        Ok(Ok(data)) => json!({"ok":true,"data":data}),
        Ok(Err(e)) => json!({"ok":false,"error":format!("{e:#}")}),
        Err(_) => json!({"ok":false,"error":"Internal error. The operation could not finish."}),
    };
    CString::new(value.to_string()).unwrap().into_raw()
}
/// Release a response allocated by `devkit_request`.
///
/// # Safety
/// `pointer` must be null or an unmodified pointer returned by `devkit_request`
/// that has not already been freed. The response must no longer be in use.
#[no_mangle]
pub unsafe extern "C" fn devkit_free(pointer: *mut c_char) {
    if !pointer.is_null() {
        drop(CString::from_raw(pointer));
    }
}

#[cfg(test)]
mod installation_tests {
    use super::*;

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
