use crate::Context;
use anyhow::{bail, Result};
use rusqlite::{params, Connection, TransactionBehavior};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const RECENT_LIMIT: u32 = 100;
const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS reading_entries (
        sequence INTEGER PRIMARY KEY,
        kind TEXT NOT NULL,
        document_id TEXT NOT NULL,
        title TEXT NOT NULL,
        pack_id TEXT NOT NULL,
        stack TEXT NOT NULL,
        version TEXT NOT NULL,
        source_url TEXT NOT NULL,
        timestamp INTEGER NOT NULL,
        anchor TEXT,
        focus_text TEXT,
        UNIQUE(kind, document_id)
    );
    CREATE INDEX IF NOT EXISTS reading_order ON reading_entries(kind, sequence DESC);
";

fn database(ctx: &Context) -> Result<Connection> {
    let db = ctx.database()?;
    db.execute_batch(SCHEMA)?;
    Ok(db)
}

fn snapshot(db: &Connection) -> Result<Value> {
    let mut statement = db.prepare(
        "SELECT r.kind, r.document_id, r.title, r.pack_id, r.stack, r.version, r.source_url,
                r.timestamp, r.anchor, r.focus_text, d.id IS NOT NULL, other.timestamp
         FROM reading_entries r
         LEFT JOIN documents d ON d.id=r.document_id
         LEFT JOIN reading_entries other
           ON other.document_id=r.document_id
           AND other.kind=CASE r.kind WHEN 'saved' THEN 'recent' ELSE 'saved' END
         ORDER BY r.kind, r.sequence DESC",
    )?;
    let mut saved = Vec::new();
    let mut recent = Vec::new();
    let rows = statement.query_map([], |row| {
        let is_saved = row.get_ref(0)?.as_str()? == "saved";
        let timestamp = Some(row.get::<_, u64>(7)?);
        let other_timestamp = row.get::<_, Option<u64>>(11)?;
        let raw_title = row.get::<_, String>(2)?;
        let pack = row.get::<_, String>(3)?;
        let item = json!({
            "id": row.get::<_, String>(1)?,
            "title": crate::doc_search::plain_title(&raw_title, &pack),
            "pack_id": pack,
            "stack": row.get::<_, String>(4)?,
            "version": row.get::<_, String>(5)?,
            "source_url": row.get::<_, String>(6)?,
            "visited_at": if is_saved { other_timestamp } else { timestamp },
            "saved_at": if is_saved { timestamp } else { other_timestamp },
            "anchor": row.get::<_, Option<String>>(8)?,
            "focus_text": row.get::<_, Option<String>>(9)?,
            "installed": row.get::<_, bool>(10)?,
        });
        Ok((is_saved, item))
    })?;
    for row in rows {
        let (is_saved, item) = row?;
        if is_saved {
            saved.push(item);
        } else {
            recent.push(item);
        }
    }
    Ok(json!({"saved":saved,"recent":recent}))
}

fn remember(
    db: &Connection,
    kind: &str,
    id: &str,
    anchor: Option<&str>,
    focus_text: Option<&str>,
) -> Result<()> {
    let anchor = anchor.filter(|value| !value.is_empty());
    if anchor.is_some_and(|value| value.len() > 4096 || value.chars().any(char::is_control)) {
        bail!("Invalid article anchor");
    }
    let focus_text: Option<String> = focus_text
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.chars().take(512).collect());
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let affected = db.execute(
        "INSERT OR REPLACE INTO reading_entries (
            kind, document_id, title, pack_id, stack, version, source_url, timestamp, anchor, focus_text
         )
         SELECT ?1, d.id, d.title, d.pack_id, d.stack, d.version, d.source_url, ?3, ?4, ?5
         FROM documents d WHERE d.id=?2",
        params![kind, id, now, anchor, focus_text],
    )?;
    if affected == 0 {
        bail!("This article is not installed. Search installed documentation for an exact article ID.");
    }
    if kind == "recent" {
        db.execute(
            "DELETE FROM reading_entries WHERE kind='recent' AND sequence IN (
                 SELECT sequence FROM reading_entries WHERE kind='recent'
                 ORDER BY sequence DESC LIMIT -1 OFFSET ?1
             )",
            [RECENT_LIMIT],
        )?;
    }
    Ok(())
}

fn record(
    ctx: &Context,
    kind: &str,
    id: &str,
    anchor: Option<&str>,
    focus_text: Option<&str>,
) -> Result<Value> {
    let mut db = database(ctx)?;
    let transaction = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    remember(&transaction, kind, id, anchor, focus_text)?;
    let result = snapshot(&transaction)?;
    transaction.commit()?;
    Ok(result)
}

pub fn list(ctx: &Context) -> Result<Value> {
    snapshot(&database(ctx)?)
}

pub fn visit(
    ctx: &Context,
    id: &str,
    anchor: Option<&str>,
    focus_text: Option<&str>,
) -> Result<Value> {
    record(ctx, "recent", id, anchor, focus_text)
}

pub fn save(
    ctx: &Context,
    id: &str,
    anchor: Option<&str>,
    focus_text: Option<&str>,
) -> Result<Value> {
    record(ctx, "saved", id, anchor, focus_text)
}

pub fn remove(ctx: &Context, id: &str) -> Result<Value> {
    let mut db = database(ctx)?;
    let transaction = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "DELETE FROM reading_entries WHERE kind='saved' AND document_id=?1",
        [id],
    )?;
    let result = snapshot(&transaction)?;
    transaction.commit()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_keeps_latest_unique_entries_without_changing_saved_positions() -> Result<()> {
        let db = Connection::open_in_memory()?;
        db.execute_batch(
            "CREATE TABLE documents(id TEXT PRIMARY KEY, title TEXT, pack_id TEXT,
                 stack TEXT, version TEXT, source_url TEXT);",
        )?;
        db.execute_batch(SCHEMA)?;
        for n in 0..=RECENT_LIMIT {
            let id = format!("go@1/test-{n}");
            db.execute(
                "INSERT INTO documents VALUES (?1, 'Example', 'go', 'go', '1', 'https://go.dev')",
                [&id],
            )?;
            remember(&db, "recent", &id, Some("intro"), None)?;
            if n == 0 {
                remember(
                    &db,
                    "saved",
                    &id,
                    Some("saved-anchor"),
                    Some("Saved section"),
                )?;
            }
        }
        let result = snapshot(&db)?;
        assert_eq!(
            result["recent"].as_array().unwrap().len(),
            RECENT_LIMIT as usize
        );
        assert_eq!(
            result["recent"][0]["id"],
            format!("go@1/test-{RECENT_LIMIT}")
        );
        assert_eq!(result["saved"][0]["anchor"], "saved-anchor");
        assert!(result["saved"][0]["visited_at"].is_null());
        remember(&db, "recent", "go@1/test-0", Some("return"), None)?;
        db.execute("DELETE FROM documents WHERE id='go@1/test-0'", [])?;
        let result = snapshot(&db)?;
        assert_eq!(result["recent"][0]["anchor"], "return");
        assert_eq!(result["saved"][0]["focus_text"], "Saved section");
        assert_eq!(result["saved"][0]["installed"], false);
        assert!(result["saved"][0]["visited_at"].is_number());
        Ok(())
    }
}
