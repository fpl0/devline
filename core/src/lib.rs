use anyhow::{bail, Context as ErrorContext, Result};
use fs2::FileExt;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    ffi::{CStr, CString},
    fs,
    os::raw::c_char,
    path::{Path, PathBuf},
    time::Duration,
};
mod doc_index;
mod doc_links;
mod doc_search;
mod markdown;
mod reference;
pub mod templates;
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
    pub test_command: String,
    #[serde(default)]
    pub docs: Vec<String>,
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
        let mut db = Connection::open(self.home.join("library.sqlite3"))?;
        db.busy_timeout(Duration::from_secs(10))?;
        db.pragma_update(None, "journal_mode", "WAL")?;
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
fn install_docs(ctx: &Context, id: &str, version: Option<&str>) -> Result<Value> {
    let pack = ctx
        .catalog
        .docs
        .iter()
        .filter(|p| p.id == id && version.is_none_or(|v| v == p.version))
        .max_by(|a, b| updates::compare_doc_releases(&a.version, &b.version));
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
    let _lock = lock_library(ctx)?;
    let directory = safe_catalog_path(ctx, &pack.path)?;
    let shards = pack.shards.unwrap_or(1);
    if !(1..=1024).contains(&shards) {
        bail!("Invalid documentation pack shard count");
    }
    let mut db = ctx.database()?;
    let previous: Option<(String, usize)> = db
        .query_row(
            "SELECT digest,count FROM packs WHERE id=?1 AND version=?2",
            params![pack.id, pack.version],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let tx = db.transaction()?;
    let mut hash = Sha256::new();
    let mut count = 0;
    {
        let mut document = tx.prepare("INSERT INTO documents(id,pack_id,title,body,html,stack,version,source_url,category) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)")?;
        let mut fts =
            tx.prepare("INSERT INTO docs_fts(rowid,title,body,stack) VALUES(?1,?2,?3,?4)")?;
        let mut aliases = tx.prepare("INSERT INTO aliases VALUES(?1,?2)")?;
        for shard in 1..=shards {
            let file = if shard == 1 {
                "documents.json".to_owned()
            } else {
                format!("documents-{shard}.json")
            };
            let bytes = fs::read(directory.join(&file))
                .with_context(|| format!("Read {file} for {}@{}", pack.id, pack.version))?;
            if shards > 1 {
                hash.update((bytes.len() as u64).to_le_bytes());
            }
            hash.update(&bytes);
            if previous.is_some() {
                continue;
            }
            let docs: Vec<Document> = serde_json::from_slice(&bytes)
                .with_context(|| format!("Parse {file} for {}@{}", pack.id, pack.version))?;
            drop(bytes);
            if docs.is_empty() {
                bail!("Documentation pack {file} is empty");
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
                    doc_index::add_alias(&tx, &id, alias)?;
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
        return Ok(json!({"id":pack.id,"version":pack.version,"count":count}));
    }
    tx.execute(
        "INSERT INTO packs VALUES(?1,?2,?3,?4)",
        params![pack.id, pack.version, digest, count],
    )?;
    doc_index::mark_pack(&tx, &pack.id, &pack.version)?;
    tx.commit()?;
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
            let mut value = serde_json::to_value(pack)?;
            value["installed"] = json!(present);
            Ok(value)
        })
        .collect()
}
pub fn project_context(path: &Path) -> Value {
    let mut current = path.to_path_buf();
    let markers = [
        ("go.mod", "go"),
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python"),
        ("requirements.txt", "python"),
        ("Gemfile", "ruby"),
        ("pom.xml", "java"),
        ("mix.exs", "elixir"),
        ("global.json", "dotnet"),
    ];
    loop {
        let mut stacks = std::collections::BTreeSet::new();
        for (file, stack) in markers {
            if current.join(file).is_file() {
                stacks.insert(stack.to_string());
            }
        }
        if let Ok(entries) = fs::read_dir(&current) {
            if entries
                .flatten()
                .any(|e| e.path().extension().is_some_and(|v| v == "csproj"))
            {
                stacks.insert("dotnet".into());
            }
        }
        if !stacks.is_empty() {
            return json!({"path": current, "stacks": stacks, "stack": if stacks.len()==1 {stacks.first().cloned()} else {None}});
        }
        if !current.pop() {
            return json!({"path":path,"stacks":[],"stack":null});
        }
    }
}

pub fn execute(request: &Value) -> Result<Value> {
    let ctx = Context::new(request)?;
    match text_arg(request, "command")? {
        "project.context" => Ok(project_context(Path::new(text_arg(request, "path")?))),
        "catalog" => {
            let db = ctx.database()?;
            let docs = catalog_docs(&ctx, &db)?;
            Ok(json!({"docs":docs,"templates":templates::list(&ctx)?,"home":ctx.home}))
        }
        "docs.install" => install_docs(&ctx, text_arg(request, "id")?, request["version"].as_str()),
        "docs.search" => doc_search::search(&ctx, request),
        "docs.show" => doc_search::show(&ctx, request),
        "docs.resolve" => doc_links::resolve(&ctx, request),
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
        "docs.remove" => {
            let _lock = lock_library(&ctx)?;
            let mut db = ctx.database()?;
            let tx = db.transaction()?;
            let id = text_arg(request, "id")?;
            let version = text_arg(request, "version")?;
            tx.execute("INSERT INTO docs_fts(docs_fts,rowid,title,body,stack) SELECT 'delete',rowid,title,body,stack FROM documents WHERE pack_id=?1 AND version=?2",params![id,version])?;
            tx.execute("DELETE FROM aliases WHERE document_id IN (SELECT id FROM documents WHERE pack_id=?1 AND version=?2)",params![id,version])?;
            tx.execute(
                "DELETE FROM documents WHERE pack_id=?1 AND version=?2",
                params![id, version],
            )?;
            tx.execute(
                "DELETE FROM packs WHERE id=?1 AND version=?2",
                params![id, version],
            )?;
            tx.commit()?;
            Ok(json!({"removed":true}))
        }
        "templates.list" => templates::list(&ctx),
        "runtime.list" => templates::runtimes(&ctx),
        "updates.status" => updates::status(&ctx),
        "updates.check" => updates::check(&ctx),
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
        "prepare" => {
            let id = text_arg(request, "id")?;
            let template = templates::install(&ctx, id)?;
            let releases = templates::list(&ctx)?;
            let entry = releases
                .as_array()
                .context("Invalid template library")?
                .iter()
                .find(|t| t["id"] == template["id"] && t["version"] == template["version"])
                .context("Unknown template")?;
            let mut docs = vec![];
            for doc in entry["docs"]
                .as_array()
                .context("Missing related documentation")?
            {
                let reference = doc.as_str().context("Invalid documentation reference")?;
                let (pack, version) = reference
                    .split_once('@')
                    .map_or((reference, None), |(id, version)| (id, Some(version)));
                docs.push(install_docs(&ctx, pack, version)?);
            }
            Ok(json!({"template":template,"docs":docs,"doctor":templates::doctor(&ctx,id)?}))
        }
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
