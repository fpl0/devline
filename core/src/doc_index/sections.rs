use super::symbol_key;
use crate::{
    doc_search::{preceding_markdown, related_root},
    markdown, Context,
};
use anyhow::Result;
use rusqlite::{params, Connection, TransactionBehavior};

const UNINDEXED: &str = "FROM json_each(?1) r JOIN packs p
    ON p.id=substr(r.value,1,instr(r.value,'@')-1)
    AND p.version=substr(r.value,instr(r.value,'@')+1)
    WHERE NOT EXISTS (
        SELECT 1 FROM doc_section_packs i WHERE i.id=p.id AND i.version=p.version)";

pub(in crate::doc_index) fn ensure(
    db: &mut Connection,
    ctx: &Context,
    releases: &str,
) -> Result<()> {
    let initialized = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='doc_section_packs')",
        [],
        |row| row.get::<_, bool>(0),
    )?;
    if initialized {
        let pending = db.query_row(
            &format!("SELECT EXISTS(SELECT 1 {UNINDEXED})"),
            [releases],
            |row| row.get::<_, bool>(0),
        )?;
        if !pending {
            return Ok(());
        }
    }

    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_schema(&tx)?;
    let packs = {
        let mut stmt = tx.prepare(&format!("SELECT p.id,p.version {UNINDEXED}"))?;
        let rows = stmt.query_map([releases], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    for (id, version) in packs {
        let rules = ctx
            .catalog
            .docs
            .iter()
            .find(|pack| pack.id == id && pack.version == version)
            .and_then(|pack| pack.markdown.as_deref());
        index_pack(&tx, &id, &version, rules)?;
    }
    tx.commit()?;
    Ok(())
}

pub(in crate::doc_index) fn ensure_schema(db: &Connection) -> Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS doc_sections(
            rowid INTEGER PRIMARY KEY, document_rowid INTEGER NOT NULL,
            heading TEXT NOT NULL, expanded TEXT NOT NULL, anchor TEXT);
         CREATE INDEX IF NOT EXISTS doc_sections_document ON doc_sections(document_rowid);
         CREATE TABLE IF NOT EXISTS doc_section_packs(id TEXT,version TEXT,PRIMARY KEY(id,version));
         CREATE VIRTUAL TABLE IF NOT EXISTS doc_sections_fts USING fts5(
            heading,expanded,content='doc_sections',content_rowid='rowid',
            tokenize='porter unicode61 remove_diacritics 2');
         CREATE TRIGGER IF NOT EXISTS doc_section_insert AFTER INSERT ON doc_sections BEGIN
            INSERT INTO doc_sections_fts(rowid,heading,expanded)
                VALUES(new.rowid,new.heading,new.expanded);
         END;
         CREATE TRIGGER IF NOT EXISTS doc_section_delete AFTER DELETE ON doc_sections BEGIN
            INSERT INTO doc_sections_fts(doc_sections_fts,rowid,heading,expanded)
                VALUES('delete',old.rowid,old.heading,old.expanded);
         END;
         CREATE TRIGGER IF NOT EXISTS doc_section_remove_document AFTER DELETE ON documents BEGIN
            DELETE FROM doc_sections WHERE document_rowid=old.rowid;
         END;
         CREATE TRIGGER IF NOT EXISTS doc_section_remove_pack AFTER DELETE ON packs BEGIN
            DELETE FROM doc_section_packs WHERE id=old.id AND version=old.version;
         END;",
    )?;
    Ok(())
}

pub(in crate::doc_index) fn index_pack(
    db: &Connection,
    id: &str,
    version: &str,
    rules: Option<&[String]>,
) -> Result<()> {
    let mut docs = db.prepare_cached(
        "SELECT rowid,id,html,CASE WHEN html IS NULL THEN body END
         FROM documents WHERE pack_id=?1 AND version=?2",
    )?;
    let mut insert = db.prepare_cached(
        "INSERT INTO doc_sections(document_rowid,heading,expanded,anchor)
         VALUES(?1,?2,?3,?4)",
    )?;
    let mut rows = docs.query(params![id, version])?;
    while let Some(row) = rows.next()? {
        let rowid = row.get::<_, i64>(0)?;
        let mut index = |heading: &str, anchor: Option<&str>| -> Result<()> {
            if heading.is_empty()
                || heading.len() > 512
                || id == crate::apple_docs::PACK_ID && heading == "SDK provenance"
            {
                return Ok(());
            }
            let expanded = expand_identifiers(heading);
            insert.execute(params![
                rowid,
                heading,
                if expanded == heading { "" } else { &expanded },
                anchor,
            ])?;
            Ok(())
        };
        if let Some(html) = row.get_ref(2)?.as_str_or_null()? {
            for (heading, anchor) in html_headings(searchable_html(id, html)).take(1024) {
                index(&heading, anchor.as_deref())?;
            }
        } else {
            let document_id = row.get_ref(1)?.as_str()?;
            let root = related_root(document_id);
            let path = root.split_once('/').map_or("", |(_, path)| path);
            let body = row.get_ref(3)?.as_str_or_null()?;
            if let Some(body) = body.filter(|_| markdown::matches(id, path, rules)) {
                let preceding = if root != document_id {
                    preceding_markdown(db, document_id)?
                } else {
                    String::new()
                };
                let source = markdown::with_continuation(body, &preceding);
                for (heading, anchor) in markdown::headings(&source).take(1024) {
                    index(&heading, anchor.as_deref())?;
                }
            }
        }
    }
    db.execute(
        "INSERT INTO doc_section_packs(id,version) VALUES(?1,?2)",
        params![id, version],
    )?;
    Ok(())
}

pub(in crate::doc_index) fn focus(
    db: &Connection,
    id: &str,
    alias: &str,
) -> Result<Option<(String, Option<String>)>> {
    let Some(key) = symbol_key(alias) else {
        return Ok(None);
    };
    let mut stmt = db.prepare_cached("SELECT html FROM documents WHERE id=?1")?;
    let mut rows = stmt.query([id])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let Some(html) = row.get_ref(0)?.as_str_or_null()? else {
        return Ok(None);
    };
    let leaf = key.rsplit_once('.').map(|(_, leaf)| leaf);
    let mut fallback = None;
    for (heading, anchor) in html_headings(html) {
        // A heading without an anchor still names the section to show.
        let candidate = symbol_key(&heading);
        let anchored = anchor.as_deref().and_then(symbol_key);
        if candidate.as_deref() == Some(&key) || anchored.as_deref() == Some(&key) {
            return Ok(Some((heading, anchor)));
        }
        if fallback.is_none() && leaf.is_some() && candidate.as_deref() == leaf {
            fallback = Some((heading, anchor));
        }
    }
    Ok(fallback)
}

fn searchable_html<'a>(pack: &str, html: &'a str) -> &'a str {
    if !matches!(pack, "rust" | "axum") {
        return html;
    }
    // Rustdoc repeats compiler-generated impls on every type page. They
    // follow the authored documentation and remain available in the reader.
    html.split_once("<h2 id=\"synthetic-implementations\"")
        .or_else(|| html.split_once("<h2 id=\"blanket-implementations\""))
        .map_or(html, |(authored, _)| authored)
}

pub(in crate::doc_index) fn expand_identifiers(value: &str) -> String {
    let mut expanded = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    let mut previous = ' ';
    while let Some(ch) = chars.next() {
        let next = chars.peek().copied().unwrap_or(' ');
        let camel_boundary = ch.is_uppercase()
            && (previous.is_lowercase()
                || previous.is_numeric()
                || previous.is_uppercase() && next.is_lowercase());
        if camel_boundary {
            expanded.push(' ');
        }
        expanded.push(if ch == '_' { ' ' } else { ch });
        previous = ch;
    }
    expanded
}

// Pack HTML is an escaped fragment generated by Devline's importers. Read its
// heading tags and text without constructing a full tree for each article.
fn html_headings(html: &str) -> impl Iterator<Item = (String, Option<String>)> + '_ {
    let mut active: Option<(&str, String, Option<String>)> = None;
    let mut section_id = None;
    html.split('<').skip(1).filter_map(move |fragment| {
        let (tag, following_text) = fragment.split_once('>')?;
        let name = tag.split_ascii_whitespace().next().unwrap_or(tag);
        let outer_anchor = section_id.take();
        if name == "section" && following_text.trim().is_empty() {
            section_id = html_id(tag);
        }
        if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "dt") {
            let anchor = html_id(tag).or(outer_anchor);
            active = Some((name, following_text.to_owned(), anchor));
            return None;
        }
        let (kind, heading, anchor) = active.as_mut()?;
        if name.strip_prefix('/') == Some(*kind) {
            let (kind, heading, anchor) = active.take()?;
            if kind == "dt" && anchor.is_none() {
                return None;
            }
            let heading = decode_entities(&heading);
            let heading = heading
                .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, '¶' | '§'))
                .trim_end_matches(|ch: char| ch.is_whitespace() || ch == '¶')
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            return Some((heading, anchor));
        }
        if anchor.is_none() && matches!(name, "a" | "span" | "code") {
            *anchor = html_id(tag);
        }
        if name == "br" {
            heading.push(' ');
        }
        heading.push_str(following_text);
        if heading.len() > 2048 {
            active = None;
        }
        None
    })
}

fn html_id(tag: &str) -> Option<String> {
    // Importers serialize escaped attributes in double quotes. Split all
    // attributes together so text inside an unrelated attribute is ignored.
    let mut parts = tag.split('"');
    while let (Some(name), Some(value)) = (parts.next(), parts.next()) {
        if name.ends_with(" id=") {
            let anchor = decode_entities(value);
            return (!anchor.is_empty()
                && anchor.len() <= 512
                && !anchor.chars().any(char::is_control))
            .then_some(anchor);
        }
    }
    None
}

fn decode_entities(mut text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    while let Some((before, after)) = text.split_once('&') {
        decoded.push_str(before);
        let end = after.floor_char_boundary(after.len().min(16));
        if let Some((entity, _)) = after[..end].split_once(';') {
            let value = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                "ndash" => Some('–'),
                "mdash" => Some('—'),
                "para" => Some('¶'),
                "sect" => Some('§'),
                _ => entity.strip_prefix('#').and_then(|numeric| {
                    let code = match numeric.strip_prefix(['x', 'X']) {
                        Some(hex) => u32::from_str_radix(hex, 16),
                        None => numeric.parse(),
                    };
                    code.ok().and_then(char::from_u32)
                }),
            };
            if let Some(value) = value {
                decoded.push(value);
                text = &after[entity.len() + 1..];
                continue;
            }
        }
        decoded.push('&');
        text = after;
    }
    decoded.push_str(text);
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{templates::Stage, Catalog};

    #[test]
    fn extracts_headings_and_declaration_anchors_from_pack_markup() {
        let html = r#"<h2 id="main">Use &amp; reuse</h2>
            <section id="more"><h3>Use <code>HTTP</code>&#182;</h3></section>
            <h4><code>fs.readFile(path)</code><a id="fs_readfile"></a></h4>
            <dt id="Foo&lt;T&gt;"><code>Foo&lt;T&gt;</code></dt>
            <pre>&lt;h2 id="not-heading"&gt;Example&lt;/h2&gt;</pre>
            <dt>Unanchored label</dt>"#;
        assert_eq!(
            html_headings(html).collect::<Vec<_>>(),
            [
                ("Use & reuse".into(), Some("main".into())),
                ("Use HTTP".into(), Some("more".into())),
                ("fs.readFile(path)".into(), Some("fs_readfile".into())),
                ("Foo<T>".into(), Some("Foo<T>".into())),
            ]
        );
        assert_eq!(
            expand_identifiers("JSONEncoder.read_text"),
            "JSON Encoder.read text"
        );
    }

    #[test]
    fn backfills_existing_releases_and_drops_sections_when_they_are_removed() -> Result<()> {
        let stage = Stage::new(&std::env::temp_dir())?;
        let ctx = Context {
            home: stage.0.clone(),
            catalog_dir: stage.0.clone(),
            catalog: Catalog {
                schema_version: 1,
                docs: Vec::new(),
                templates: Vec::new(),
            },
        };
        let mut db = ctx.database()?;
        db.execute_batch(
            "INSERT INTO packs VALUES('fastapi','1','digest',2),('fastapi','2','digest',1);
             INSERT INTO documents (id,pack_id,version,body) VALUES
             ('fastapi@1/setup.md','fastapi','1','# Setup

```python
first = 1'),
             ('fastapi@1/setup.md::part-2','fastapi','1','# Hidden code heading
```

## **Valid** heading {#exact}'),
             ('fastapi@2/setup.md','fastapi','2','# Different release');",
        )?;
        ensure(&mut db, &ctx, r#"["fastapi@1"]"#)?;
        let query = |db: &Connection, term: &str| -> Result<Vec<String>> {
            let mut stmt =
                db.prepare("SELECT heading FROM doc_sections_fts WHERE doc_sections_fts MATCH ?1")?;
            let headings = stmt
                .query_map([term], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(headings)
        };
        assert!(query(&db, "hidden")?.is_empty());
        assert_eq!(query(&db, "valid")?, ["Valid heading"]);
        assert!(query(&db, "different")?.is_empty());
        let anchor: String = db.query_row(
            "SELECT s.anchor FROM doc_sections_fts
             JOIN doc_sections s ON s.rowid=doc_sections_fts.rowid
             WHERE doc_sections_fts MATCH 'valid'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(anchor, "exact");
        ensure(&mut db, &ctx, r#"["fastapi@1","fastapi@2"]"#)?;
        assert_eq!(query(&db, "different")?, ["Different release"]);

        db.execute_batch(
            "DELETE FROM documents WHERE pack_id='fastapi';
             DELETE FROM packs WHERE id='fastapi';
             INSERT INTO packs VALUES('fastapi','1','new digest',1);
             INSERT INTO documents (id,pack_id,version,body,html) VALUES
             ('fastapi@1/setup.md','fastapi','1','New section',
              '<h2 id=\"search-key\"><code>findByKey</code></h2>
               <h2 id=\"qualified-search-key\"><code>Widget.findByKey()</code></h2>
               <dt id=\"widget.Form.save\"><code>Form.save()</code></dt>');",
        )?;
        ensure(&mut db, &ctx, r#"["fastapi@1"]"#)?;
        assert!(query(&db, "valid")?.is_empty());
        assert_eq!(
            query(&db, "\"find by key\"")?,
            ["findByKey", "Widget.findByKey()"]
        );
        assert_eq!(
            focus(&db, "fastapi@1/setup.md", "findByKey()")?,
            Some(("findByKey".into(), Some("search-key".into())))
        );
        assert_eq!(
            focus(&db, "fastapi@1/setup.md", "Widget.findByKey")?,
            Some((
                "Widget.findByKey()".into(),
                Some("qualified-search-key".into())
            ))
        );
        assert_eq!(
            focus(&db, "fastapi@1/setup.md", "Other.findByKey")?,
            Some(("findByKey".into(), Some("search-key".into())))
        );
        assert_eq!(
            focus(&db, "fastapi@1/setup.md", "widget.Form.save")?,
            Some(("Form.save()".into(), Some("widget.Form.save".into())))
        );
        Ok(())
    }
}
