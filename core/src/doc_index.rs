use anyhow::Result;
use rusqlite::{params, Connection, TransactionBehavior};

mod sections;

pub(super) fn ensure_sections(
    db: &mut Connection,
    ctx: &crate::Context,
    releases: &str,
) -> Result<()> {
    sections::ensure(db, ctx, releases)
}

pub(super) fn index_pack_sections(
    tx: &rusqlite::Transaction<'_>,
    pack: &crate::DocPack,
) -> Result<()> {
    sections::ensure_schema(tx)?;
    sections::index_pack(tx, &pack.id, &pack.version, pack.markdown.as_deref())
}

pub(super) fn section_focus(
    db: &Connection,
    id: &str,
    alias: &str,
) -> Result<Option<(String, Option<String>)>> {
    sections::focus(db, id, alias)
}

pub(super) fn without_call(reference: &str) -> &str {
    let reference = reference.trim().trim_matches('`');
    if let Some((name, _)) = reference.split_once('(').filter(|(name, _)| {
        reference.ends_with(')') && !name.trim_end().contains(char::is_whitespace)
    }) {
        name.trim_end()
    } else {
        reference
    }
}

pub(super) fn symbol_key(reference: &str) -> Option<String> {
    let reference = without_call(reference);
    let (name, arity) = reference
        .rsplit_once('/')
        .map_or((reference, None), |(name, arity)| (name, Some(arity)));
    if arity
        .is_some_and(|arity| arity.is_empty() || !arity.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    let key = name.replace("::", ".").replace('#', ".");
    if !key.split('.').all(symbol_component) {
        return None;
    }
    let mut key = key.to_lowercase();
    if let Some(arity) = arity {
        key.push('/');
        key.push_str(arity);
    }
    Some(key)
}

pub(super) fn symbol_component(part: &str) -> bool {
    part.chars()
        .next()
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
        && part
            .chars()
            .all(|ch| ch.is_alphanumeric() || "_!?".contains(ch))
        || !part.is_empty() && part.bytes().all(|ch| b"<>|=+-*/%~^!&".contains(&ch))
}

pub(super) fn add_alias(db: &Connection, id: &str, alias: &str) -> Result<()> {
    let Some(key) = symbol_key(alias) else {
        return Ok(());
    };
    let mut stmt = db.prepare_cached(
        "INSERT INTO doc_symbol_keys(key,document_id,alias,depth) VALUES(?1,?2,?3,?4)
         ON CONFLICT(key,document_id) DO UPDATE SET alias=excluded.alias,depth=excluded.depth
         WHERE excluded.depth<doc_symbol_keys.depth",
    )?;
    let label = without_call(alias);
    stmt.execute(params![key, id, label, 0])?;
    let dot_count = key.bytes().filter(|byte| *byte == b'.').count();
    for (depth, (index, _)) in key
        .match_indices('.')
        .enumerate()
        .skip(dot_count.saturating_sub(3))
    {
        stmt.execute(params![&key[index + 1..], id, label, depth + 1])?;
    }
    if let Some((base, _)) = key.split_once('/') {
        let leaf = base.rsplit('.').next().unwrap_or(base);
        stmt.execute(params![base, id, label, 1])?;
        stmt.execute(params![leaf, id, label, dot_count + 1])?;
    }
    Ok(())
}

pub(super) fn mark_pack(db: &Connection, pack: &str, version: &str) -> Result<()> {
    db.execute(
        "INSERT INTO doc_symbol_packs(id,version) VALUES(?1,?2)",
        params![pack, version],
    )?;
    Ok(())
}

pub(super) fn valid_category(category: &str) -> bool {
    matches!(category, "guide" | "api" | "compatibility" | "reference")
}

pub(super) fn category(pack: &str, record: &str, source: &str) -> &'static str {
    let source_path = source
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('/'))
        .and_then(|(_, path)| path.split(['?', '#']).next())
        .unwrap_or_default();
    let sections = || record.split('/').chain(source_path.split('/'));
    if sections().any(|part| matches!(part, "compatibility" | "breaking-changes")) {
        return "compatibility";
    }
    let section = record.split('/').next().unwrap_or(record);
    let basename = record.rsplit('/').next().unwrap_or(record);
    if matches!(section, "api" | "method" | "std")
        || matches!(pack, "java" | "axum")
        || pack == "python" && section == "library"
        || pack == "fastapi" && section == "reference"
        || matches!(pack, "elixir" | "phoenix" | "ecto")
            && (basename.chars().next().is_some_and(char::is_uppercase)
                || basename == "api-reference.html")
    {
        return "api";
    }
    if matches!(pack, "rails")
        || sections().any(|part| {
            matches!(
                part,
                "tutorial"
                    | "tutorials"
                    | "guide"
                    | "guides"
                    | "how-to"
                    | "howto"
                    | "book"
                    | "programming-guide"
            )
        })
        || pack == "fastapi" && matches!(section, "advanced" | "deployment")
    {
        return "guide";
    }
    "reference"
}

fn categorize_legacy(db: &Connection) -> Result<()> {
    let mut query =
        db.prepare("SELECT id,pack_id,source_url FROM documents WHERE category IS NULL")?;
    let rows = query
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut update = db.prepare("UPDATE documents SET category=?1 WHERE id=?2")?;
    for (id, pack, source) in rows {
        let record = id.split_once('/').map_or(id.as_str(), |(_, record)| record);
        update.execute(params![category(&pack, record, &source), id])?;
    }
    Ok(())
}

pub(super) fn ensure(db: &mut Connection) -> Result<()> {
    let initialized = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='doc_symbol_packs' AND type='table')",
        [],
        |row| row.get::<_, bool>(0),
    )?;
    if !initialized {
        let schema = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        schema.execute_batch(
            "CREATE TABLE IF NOT EXISTS doc_symbol_keys(
             key TEXT NOT NULL, document_id TEXT NOT NULL, alias TEXT NOT NULL, depth INTEGER NOT NULL,
             PRIMARY KEY(key,document_id)) WITHOUT ROWID;
         CREATE INDEX IF NOT EXISTS doc_symbol_document ON doc_symbol_keys(document_id);
         CREATE TABLE IF NOT EXISTS doc_symbol_packs(id TEXT,version TEXT,PRIMARY KEY(id,version));
         CREATE INDEX IF NOT EXISTS documents_source ON documents(source_url);
         CREATE TRIGGER IF NOT EXISTS doc_symbol_remove_document AFTER DELETE ON documents BEGIN
             DELETE FROM doc_symbol_keys WHERE document_id=old.id;
         END;
         CREATE TRIGGER IF NOT EXISTS doc_symbol_remove_pack AFTER DELETE ON packs BEGIN
             DELETE FROM doc_symbol_packs WHERE id=old.id AND version=old.version;
         END;",
        )?;
        schema.commit()?;
    }
    let category_index = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='documents_category' AND type='index')",
        [], |row| row.get::<_, bool>(0),
    )?;
    if !category_index {
        let migration = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let category_column = migration.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('documents') WHERE name='category')",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if !category_column {
            migration.execute("ALTER TABLE documents ADD COLUMN category TEXT", [])?;
        }
        categorize_legacy(&migration)?;
        migration.execute(
            "CREATE INDEX IF NOT EXISTS documents_category ON documents(pack_id,version,category)",
            [],
        )?;
        migration.commit()?;
    }
    // A marker per pack also handles imports made by earlier Devline binaries.
    const UNINDEXED: &str = "FROM packs p WHERE NOT EXISTS (
        SELECT 1 FROM doc_symbol_packs i WHERE i.id=p.id AND i.version=p.version)";
    let pending = db.query_row(&format!("SELECT EXISTS(SELECT 1 {UNINDEXED})"), [], |row| {
        row.get::<_, bool>(0)
    })?;
    if !pending {
        return Ok(());
    }
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    {
        let mut packs = tx.prepare(&format!("SELECT p.id,p.version {UNINDEXED}"))?;
        let packs = packs
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if !packs.is_empty() {
            // Older binaries omit this nullable column on subsequent imports.
            categorize_legacy(&tx)?;
            let mut aliases = tx.prepare(
                "SELECT a.document_id,a.alias FROM aliases a JOIN documents d ON a.document_id=d.id
                 JOIN packs p ON p.id=d.pack_id AND p.version=d.version
                 WHERE NOT EXISTS (SELECT 1 FROM doc_symbol_packs i WHERE i.id=p.id AND i.version=p.version)",
            )?;
            let mut rows = aliases.query([])?;
            while let Some(row) = rows.next()? {
                add_alias(&tx, row.get_ref(0)?.as_str()?, row.get_ref(1)?.as_str()?)?;
            }
        }
        for (pack, version) in packs {
            mark_pack(&tx, &pack, &version)?;
        }
    }
    tx.commit()?;
    Ok(())
}
