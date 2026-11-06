use anyhow::{bail, Context as ErrorContext, Result};
use fs2::FileExt;
use rusqlite::{params, Connection};
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
mod markdown;
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
        let db = Connection::open(self.home.join("library.sqlite3"))?;
        db.busy_timeout(Duration::from_secs(10))?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.execute_batch("CREATE TABLE IF NOT EXISTS packs(id TEXT, version TEXT, digest TEXT NOT NULL, count INTEGER, PRIMARY KEY(id,version));
            CREATE TABLE IF NOT EXISTS documents(id TEXT PRIMARY KEY, pack_id TEXT, title TEXT, body TEXT, html TEXT, stack TEXT, version TEXT, source_url TEXT);
            CREATE TABLE IF NOT EXISTS aliases(alias TEXT, document_id TEXT);
            CREATE INDEX IF NOT EXISTS aliases_lookup ON aliases(alias);
            CREATE INDEX IF NOT EXISTS documents_pack ON documents(pack_id,version);
            CREATE VIRTUAL TABLE IF NOT EXISTS docs_fts USING fts5(title,body,stack,content='documents',content_rowid='rowid');")?;
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
fn installed(db: &Connection, id: &str, version: &str) -> bool {
    db.query_row(
        "SELECT 1 FROM packs WHERE id=?1 AND version=?2",
        params![id, version],
        |_| Ok(()),
    )
    .is_ok()
}
fn install_docs(ctx: &Context, id: &str, version: Option<&str>) -> Result<Value> {
    let pack = ctx
        .catalog
        .docs
        .iter()
        .rev()
        .find(|p| p.id == id && version.is_none_or(|v| v == p.version))
        .with_context(|| format!("Documentation pack {id} is unavailable in the catalog"))?;
    let _lock = lock_library(ctx)?;
    let bytes = fs::read(safe_catalog_path(ctx, &pack.path)?.join("documents.json"))?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let docs: Vec<Document> = serde_json::from_slice(&bytes)?;
    if docs.is_empty() {
        bail!("The documentation pack is empty");
    }
    let mut db = ctx.database()?;
    let previous = db.query_row(
        "SELECT digest FROM packs WHERE id=?1 AND version=?2",
        params![pack.id, pack.version],
        |r| r.get::<_, String>(0),
    );
    if let Ok(old) = previous {
        if old != digest {
            bail!(
                "The content of {}@{} changed. Publish a new pack version.",
                pack.id,
                pack.version
            );
        }
        return Ok(json!({"id":pack.id,"version":pack.version,"count":docs.len()}));
    }
    let tx = db.transaction()?;
    for doc in &docs {
        if doc.id.is_empty() || doc.title.is_empty() {
            bail!("Document id and title must be present");
        }
        let id = format!("{}@{}/{}", pack.id, pack.version, doc.id);
        tx.execute("INSERT INTO documents(id,pack_id,title,body,html,stack,version,source_url) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id,pack.id,doc.title,doc.body,doc.html,pack.stack,pack.version,doc.source_url])?;
        tx.execute(
            "INSERT INTO docs_fts(rowid,title,body,stack) VALUES(?1,?2,?3,?4)",
            params![tx.last_insert_rowid(), doc.title, doc.body, pack.stack],
        )?;
        for alias in std::iter::once(&doc.title).chain(doc.aliases.iter()) {
            tx.execute(
                "INSERT INTO aliases VALUES(?1,?2)",
                params![alias.to_lowercase(), id],
            )?;
        }
    }
    tx.execute(
        "INSERT INTO packs VALUES(?1,?2,?3,?4)",
        params![pack.id, pack.version, digest, docs.len()],
    )?;
    tx.commit()?;
    Ok(json!({"id":pack.id,"version":pack.version,"count":docs.len()}))
}
struct DocScope<'a> {
    stack: &'a str,
    version: &'a str,
    pack_id: &'a str,
}

impl<'a> DocScope<'a> {
    fn new(request: &'a Value) -> Self {
        Self {
            stack: request["stack"].as_str().unwrap_or(""),
            version: request["version"].as_str().unwrap_or(""),
            pack_id: request["pack_id"].as_str().unwrap_or(""),
        }
    }
}

struct DocSummary {
    id: String,
    pack_id: String,
    title: String,
    stack: String,
    version: String,
    source_url: String,
    snippet: String,
}

fn summary(row: &rusqlite::Row) -> rusqlite::Result<DocSummary> {
    Ok(DocSummary {
        id: row.get(0)?,
        pack_id: row.get(1)?,
        title: row.get(2)?,
        stack: row.get(3)?,
        version: row.get(4)?,
        source_url: row.get(5)?,
        snippet: row.get(6)?,
    })
}

impl DocSummary {
    fn json(self, ctx: &Context, anchor: Option<String>) -> Value {
        let title = self.display_title(ctx);
        let mut value = json!({
            "id": self.id,
            "pack_id": self.pack_id,
            "title": title,
            "stack": self.stack,
            "version": self.version,
            "source_url": self.source_url,
            "snippet": self.snippet,
        });
        if let Some(anchor) = anchor {
            value["match_text"] = json!(anchor);
        }
        value
    }

    fn display_title(&self, ctx: &Context) -> String {
        let fallback = ctx
            .catalog
            .docs
            .iter()
            .find(|pack| pack.id == self.pack_id)
            .map_or(self.pack_id.as_str(), |pack| pack.name.as_str());
        plain_title(&self.title, fallback)
    }
}

fn plain_title(title: &str, fallback: &str) -> String {
    let mut plain = String::new();
    let mut rest = title;
    // Remove known presentational tags. Leave language syntax such as List<E> intact.
    while let Some((before, candidate)) = rest.split_once('<') {
        plain.push_str(before);
        if let Some((tag, after)) = candidate.split_once('>') {
            let name = tag
                .trim_start_matches('/')
                .split(|ch: char| ch.is_whitespace() || ch == '/')
                .next()
                .unwrap_or("");
            if matches!(
                name,
                "a" | "abbr"
                    | "b"
                    | "br"
                    | "code"
                    | "div"
                    | "em"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "i"
                    | "img"
                    | "p"
                    | "small"
                    | "span"
                    | "strong"
                    | "sub"
                    | "sup"
            ) {
                rest = after;
                continue;
            }
        }
        plain.push('<');
        rest = candidate;
    }
    plain.push_str(rest);
    let plain = plain.trim();
    if plain.is_empty() { fallback } else { plain }.to_owned()
}

const DOC_COLUMNS: &str = "d.id,d.pack_id,d.title,d.stack,d.version,d.source_url";
const DOC_PREVIEW: &str = "substr(d.body,1,200)";

fn related_root(id: &str) -> &str {
    id.rsplit_once("::part-")
        .filter(|(_, number)| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
        .map_or(id, |(root, _)| root)
}

fn symbol_pieces(alias: &str) -> Option<(&str, &str)> {
    let reference = alias
        .rsplit_once('/')
        .filter(|(_, arity)| !arity.is_empty() && arity.bytes().all(|b| b.is_ascii_digit()))
        .map_or(alias, |(name, _)| name);
    let reference = reference.strip_suffix("()").unwrap_or(reference);
    let mut parts = reference
        .rsplit(['.', ':', '#'])
        .filter(|part| !part.is_empty());
    let symbol = parts.next()?;
    let owner = parts.next().unwrap_or("");
    if reference.contains(char::is_whitespace)
        || !symbol
            .chars()
            .all(|ch| ch.is_alphanumeric() || "_!?".contains(ch))
    {
        return None;
    }
    Some((symbol, owner))
}

fn signature_start(line: &str, name: &str) -> bool {
    line.strip_prefix(name)
        .is_some_and(|tail| tail.starts_with(['(', '<']) || tail.starts_with(" ="))
}

fn declares_callable(line: &str, name: &str) -> bool {
    if let Some(tail) = ["fn ", "def ", "defp ", "@spec "]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
    {
        return signature_start(tail, name);
    }
    if let Some(tail) = line.strip_prefix("func ") {
        let callable = tail
            .strip_prefix('(')
            .and_then(|receiver| receiver.split_once(')'))
            .map_or(tail, |(_, callable)| callable.trim_start());
        return signature_start(callable, name);
    }
    // Javadoc method signatures include modifiers and a return type before the name.
    [
        "public ",
        "protected ",
        "static ",
        "default ",
        "abstract ",
        "@safevarargs ",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
        && line
            .split_once('(')
            .is_some_and(|(head, _)| head.split_whitespace().next_back() == Some(name))
}

struct Focus<'a> {
    line: &'a str,
    offset: usize,
    score: u8,
}

fn find_focus<'a>(body: &'a str, query: &str) -> Option<Focus<'a>> {
    let query = query.trim().to_lowercase();
    if query.chars().count() < 2 {
        return None;
    }
    let symbol = symbol_pieces(&query);
    let owner_signature = symbol
        .filter(|(_, owner)| !owner.is_empty())
        .map(|(symbol, owner)| format!("{owner}.{symbol}"));
    let mut best: Option<Focus<'_>> = None;
    let mut offset = 0;
    for raw in body.split_inclusive('\n') {
        let line = raw.trim();
        let start = offset + raw.len() - raw.trim_start().len();
        offset += raw.len();
        if line.is_empty() || line.len() > 1024 {
            continue;
        }
        let lower = line.to_lowercase();
        let mut score = u8::from(lower.contains(&query)) * 20;
        if let Some((name, _)) = symbol {
            let declaration = lower
                .strip_prefix("pub ")
                .or_else(|| {
                    lower
                        .strip_prefix("pub(")?
                        .split_once(')')
                        .map(|(_, tail)| tail.trim_start())
                })
                .unwrap_or(&lower);
            let declared_type = [
                "type ",
                "class ",
                "struct ",
                "enum ",
                "interface ",
                "trait ",
                "module ",
            ]
            .iter()
            .find_map(|prefix| declaration.strip_prefix(prefix))
            .and_then(|tail| tail.split([' ', '(', '<', '{', '\t']).next())
            .and_then(|qualified| qualified.rsplit(['.', ':']).next());
            if declared_type == Some(name) {
                score = 120;
            } else if declares_callable(declaration, name) {
                score = 115;
            } else if owner_signature
                .as_deref()
                .is_some_and(|qualified| signature_start(declaration, qualified))
                || signature_start(declaration, &query)
            {
                score = 100;
            } else if signature_start(declaration, name) {
                score = 85;
            }
        }
        if score > 0 && best.as_ref().is_none_or(|old| score > old.score) {
            best = Some(Focus {
                line,
                offset: start,
                score,
            });
            if score == 120 {
                break;
            }
        }
    }
    best
}

fn compact_excerpt(text: &str) -> String {
    let text: String = text.chars().take(260).collect();
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn resolve_alias(
    db: &Connection,
    original: DocSummary,
    alias: &str,
    exact: bool,
) -> Result<(DocSummary, Option<String>)> {
    let root = related_root(&original.id);
    let mut best = None;
    let mut best_score = 0;
    // The existing catalog names continuations ::part-N. Scan that indexed ID range
    // only for exact symbol aliases. The pack itself and its installed digest stay intact.
    let include_parts = exact && symbol_pieces(alias).is_some();
    let sql = format!(
        "SELECT {DOC_COLUMNS},d.body FROM documents d
         WHERE d.id=?1 OR (?2 AND d.id>=?3 AND d.id<?4)
         ORDER BY d.id<>?1, length(d.id), d.id LIMIT 16"
    );
    let mut stmt = db.prepare_cached(&sql)?;
    let mut rows = stmt.query(params![
        original.id,
        include_parts,
        format!("{root}::part-"),
        format!("{root}::part."),
    ])?;
    while let Some(row) = rows.next()? {
        let body: String = row.get(6)?;
        let Some(focus) = find_focus(&body, alias) else {
            continue;
        };
        if focus.score > best_score {
            let mut hit = summary(row)?;
            hit.snippet = compact_excerpt(&body[focus.offset..]);
            best = Some((hit, Some(focus.line.chars().take(180).collect())));
            best_score = focus.score;
            if best_score == 120 {
                break;
            }
        }
    }
    Ok(best.unwrap_or((original, None)))
}

fn alias_candidates(
    db: &Connection,
    scope: &DocScope,
    query: &str,
    prefix: bool,
    limit: i64,
) -> Result<Vec<(DocSummary, String)>> {
    // Bounds are bound parameters, so punctuation in names is literal. The existing
    // aliases_lookup index serves both equality and prefix matches.
    let upper = query
        .char_indices()
        .rev()
        .find_map(|(offset, ch)| {
            let next = u32::from(ch) + 1;
            char::from_u32(if next == 0xd800 { 0xe000 } else { next })
                .map(|next| format!("{}{next}", &query[..offset]))
        })
        .unwrap_or_default();
    let matches = if prefix && !upper.is_empty() {
        "a.alias>=?1 AND a.alias<?2"
    } else {
        "a.alias=?1"
    };
    let sql = format!(
        "SELECT {DOC_COLUMNS},{DOC_PREVIEW},MIN(a.alias) FROM aliases a
         JOIN documents d ON d.id=a.document_id
         WHERE {matches} AND (?3='' OR d.stack=?3) AND (?4='' OR d.version=?4) AND (?5='' OR d.pack_id=?5)
         GROUP BY d.id
         ORDER BY MIN(a.alias)<>?1, CASE
           WHEN instr(d.id, '/api/')>0 OR instr(d.id, '/std/')>0 THEN 0
           WHEN d.id LIKE '%/index.md' THEN 2
           ELSE 1 END,
           length(MIN(a.alias)), lower(d.title), d.id
         LIMIT ?6"
    );
    let mut stmt = db.prepare(&sql)?;
    let rows = stmt.query_map(
        params![
            query,
            upper,
            scope.stack,
            scope.version,
            scope.pack_id,
            limit
        ],
        |row| Ok((summary(row)?, row.get(7)?)),
    )?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

fn search(ctx: &Context, request: &Value) -> Result<Value> {
    let query = request["query"].as_str().unwrap_or("").trim();
    if query.len() > 512 {
        bail!("Search query must be at most 512 bytes");
    }
    let scope = DocScope::new(request);
    let limit = request["limit"].as_u64().unwrap_or(30).clamp(1, 100) as i64;
    let db = ctx.database()?;
    let mut results = Vec::new();
    if query.is_empty() {
        let offset = request["offset"].as_u64().unwrap_or(0).min(100_000) as i64;
        let sql = format!(
            "SELECT {DOC_COLUMNS},{DOC_PREVIEW} FROM documents d
             WHERE (?1='' OR d.stack=?1) AND (?2='' OR d.version=?2) AND (?3='' OR d.pack_id=?3)
             ORDER BY lower(d.title), d.id LIMIT ?4 OFFSET ?5"
        );
        let mut stmt = db.prepare(&sql)?;
        for row in stmt.query_map(
            params![scope.stack, scope.version, scope.pack_id, limit, offset],
            summary,
        )? {
            results.push(row?.json(ctx, None));
        }
        return Ok(json!(results));
    }

    let normalized = query.to_lowercase();
    let mut seen = std::collections::HashSet::new();
    let mut alias_families = std::collections::HashSet::new();
    let use_prefix = normalized.chars().any(char::is_alphanumeric);
    for (hit, alias) in alias_candidates(&db, &scope, &normalized, use_prefix, limit)? {
        let (hit, anchor) = resolve_alias(&db, hit, &alias, alias == normalized)?;
        if seen.insert(hit.id.clone()) {
            alias_families.insert(related_root(&hit.id).to_owned());
            results.push(hit.json(ctx, anchor));
        }
    }
    if results.len() < limit as usize {
        let terms: Vec<_> = query
            .split_whitespace()
            .filter(|term| term.chars().any(char::is_alphanumeric))
            .take(12)
            .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
            .collect();
        if !terms.is_empty() {
            let mut expression = format!("{{title body}} : ({})", terms.join(" AND "));
            if !scope.stack.is_empty() {
                expression.push_str(&format!(
                    " AND stack : \"{}\"",
                    scope.stack.replace('"', "\"\"")
                ));
            }
            let sql = format!(
                "SELECT {DOC_COLUMNS},snippet(docs_fts,1,'','',' … ',28) FROM docs_fts
                 JOIN documents d ON d.rowid=docs_fts.rowid
                 WHERE docs_fts MATCH ?1 AND (?2='' OR d.version=?2) AND (?3='' OR d.pack_id=?3)
                 ORDER BY bm25(docs_fts,8.0,1.0,0.0), d.id LIMIT ?4"
            );
            let mut stmt = db.prepare(&sql)?;
            for row in stmt.query_map(
                params![expression, scope.version, scope.pack_id, limit * 2],
                summary,
            )? {
                let hit = row?;
                if !alias_families.contains(related_root(&hit.id)) && seen.insert(hit.id.clone()) {
                    results.push(hit.json(ctx, None));
                    if results.len() == limit as usize {
                        break;
                    }
                }
            }
        }
    }
    Ok(json!(results))
}

fn preceding_markdown(db: &Connection, id: &str) -> Result<String> {
    let root = related_root(id);
    let Some(part) = id
        .strip_prefix(root)
        .and_then(|suffix| suffix.strip_prefix("::part-"))
        .and_then(|number| number.parse::<u32>().ok())
    else {
        return Ok(String::new());
    };
    let mut stmt = db.prepare(
        "SELECT body FROM documents WHERE id=?1 OR
         (id>=?2 AND id<?3 AND CAST(substr(id,length(?1)+8) AS INTEGER) BETWEEN 2 AND ?4)
         ORDER BY CAST(substr(id,length(?1)+8) AS INTEGER)",
    )?;
    let chunks = stmt
        .query_map(
            params![
                root,
                format!("{root}::part-"),
                format!("{root}::part."),
                part.saturating_sub(1)
            ],
            |row| row.get::<_, String>(0),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(chunks.join("\n\n"))
}

fn show(ctx: &Context, request: &Value) -> Result<Value> {
    let input = text_arg(request, "id")?;
    let scope = DocScope::new(request);
    let db = ctx.database()?;
    let direct: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM documents d WHERE d.id=?1
         AND (?2='' OR d.stack=?2) AND (?3='' OR d.version=?3) AND (?4='' OR d.pack_id=?4))",
        params![input, scope.stack, scope.version, scope.pack_id],
        |row| row.get(0),
    )?;
    let (selected, anchor) = if direct {
        (input.to_owned(), None)
    } else {
        let matches = alias_candidates(&db, &scope, &input.to_lowercase(), false, 2)?;
        let mut matches = matches.into_iter();
        match (matches.next(), matches.next()) {
            (Some((hit, alias)), None) => {
                let (hit, anchor) = resolve_alias(&db, hit, &alias, true)?;
                (hit.id, anchor)
            }
            (None, None) => {
                let mut req = request.clone();
                req["query"] = json!(input);
                req["limit"] = json!(2);
                let found = search(ctx, &req)?;
                let matches = found.as_array().unwrap();
                if matches.len() != 1 {
                    bail!(
                        "Reference is ambiguous or missing. Run docs search and use a result's id."
                    );
                }
                let hit = &matches[0];
                (
                    hit["id"]
                        .as_str()
                        .context("Invalid search result")?
                        .to_owned(),
                    hit["match_text"].as_str().map(str::to_owned),
                )
            }
            _ => bail!(
                "Several documents match this reference. Use --version or an exact result id."
            ),
        }
    };
    let sql = format!("SELECT {DOC_COLUMNS},'',d.body,d.html FROM documents d WHERE d.id=?1");
    let (hit, body, mut html): (DocSummary, String, Option<String>) =
        db.query_row(&sql, [selected], |row| {
            Ok((summary(row)?, row.get(7)?, row.get(8)?))
        })?;
    let mut focus = anchor.or_else(|| {
        let query = request["query"].as_str()?;
        find_focus(&body, query).map(|focus| focus.line.chars().take(180).collect())
    });
    let rules = ctx
        .catalog
        .docs
        .iter()
        .find(|pack| pack.id == hit.pack_id && pack.version == hit.version)
        .and_then(|pack| pack.markdown.as_deref());
    let path = related_root(&hit.id)
        .split_once('/')
        .map_or("", |(_, path)| path);
    if markdown::matches(&hit.pack_id, path, rules) {
        let preceding = preceding_markdown(&db, &hit.id)?;
        let source = markdown::with_continuation(&body, &preceding);
        let (rendered, visible_focus) =
            markdown::render(&source, &hit.source_url, focus.as_deref());
        html = Some(rendered);
        focus = visible_focus;
    }
    let title = hit.display_title(ctx);
    let mut value = json!({
        "id": hit.id, "pack_id": hit.pack_id, "title": title, "stack": hit.stack,
        "version": hit.version, "source_url": hit.source_url, "body": body, "html": html,
    });
    if let Some(focus) = focus {
        value["focus_text"] = json!(focus);
    }
    Ok(value)
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
            let docs: Vec<_> = ctx
                .catalog
                .docs
                .iter()
                .map(|p| {
                    let mut v = serde_json::to_value(p).unwrap();
                    v["installed"] = json!(installed(&db, &p.id, &p.version));
                    v
                })
                .collect();
            Ok(json!({"docs":docs,"templates":templates::list(&ctx)?,"home":ctx.home}))
        }
        "docs.install" => install_docs(&ctx, text_arg(request, "id")?, request["version"].as_str()),
        "docs.search" => search(&ctx, request),
        "docs.show" => show(&ctx, request),
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
