use crate::{
    doc_search::{focus_for_link, related_root},
    markdown::web_url,
    text_arg,
    updates::{compare_doc_releases, doc_release},
    Context,
};
use anyhow::Result;
use fluent_uri::{Iri, IriRef};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::sync::OnceLock;

pub(super) const SDK_LINK_PREFIX: &str =
    "https://developer.apple.com/documentation/swiftui#devline-sdk-symbol-";

struct Location {
    id: String,
    pack: String,
    version: String,
    source: String,
}

impl Location {
    fn source_anchor(&self) -> Option<String> {
        fragment(&Iri::parse(self.source.as_str()).ok()?)
    }
}

fn fragment(url: &Iri<&str>) -> Option<String> {
    let decoded = url.fragment()?.decode().to_string().ok()?;
    (!decoded.is_empty() && !decoded.chars().any(char::is_control)).then(|| decoded.into_owned())
}

pub(super) fn resolve(ctx: &Context, request: &Value) -> Result<Value> {
    let raw = text_arg(request, "url")?.trim();
    if raw.is_empty() || raw.len() > 8192 || raw.starts_with("//") {
        return Ok(Value::Null);
    }
    let db = ctx.database()?;
    let mut pack = request["pack_id"].as_str().unwrap_or_default();
    let mut version = request["version"].as_str().unwrap_or_default();
    let origin = if let Some(id) = request["id"].as_str() {
        let row: Option<(String, String, String)> = db
            .query_row(
                "SELECT pack_id,version,source_url FROM documents WHERE id=?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some(row) = row else {
            return Ok(Value::Null);
        };
        if (!pack.is_empty() && pack != row.0) || (!version.is_empty() && version != row.1) {
            return Ok(Value::Null);
        }
        Some(row)
    } else {
        None
    };
    if let Some((id, release, _)) = &origin {
        pack = id;
        version = release;
    }
    if let Some(hash) = raw.strip_prefix(SDK_LINK_PREFIX) {
        return match &origin {
            Some((pack, version, _)) if pack == crate::apple_docs::PACK_ID => {
                resolve_sdk_symbol(&db, pack, version, hash)
            }
            _ => Ok(Value::Null),
        };
    }

    let target = if let Ok(url) = Iri::parse(raw) {
        url.normalize()
    } else {
        let Some(source) = origin
            .as_ref()
            .and_then(|row| Iri::parse(row.2.as_str()).ok())
            .filter(web_url)
        else {
            return Ok(Value::Null);
        };
        let Ok(link) = IriRef::parse(raw) else {
            return Ok(Value::Null);
        };
        let Ok(url) = link.resolve_against(&source.strip_fragment()) else {
            return Ok(Value::Null);
        };
        url.normalize()
    };
    let target = target.borrow();
    if !web_url(&target) {
        return Ok(Value::Null);
    }
    let anchor = fragment(&target);
    if target
        .fragment()
        .is_some_and(|part| !part.as_str().is_empty())
        && anchor.is_none()
    {
        return Ok(Value::Null);
    }
    let Some(page) = locate(&db, &target, pack, version)? else {
        return Ok(Value::Null);
    };
    let Some(anchor) = anchor else {
        return Ok(reference(&page.id, None, None));
    };
    let Some(html) = stored_html(&db, &page.id)? else {
        let (id, focus) = focus_for_link(&db, &page.id, &anchor)?;
        return Ok(reference(&id, Some(&anchor), focus.as_deref()));
    };
    if has_anchor(&html, &anchor) {
        return Ok(reference(&page.id, Some(&anchor), None));
    }
    // A fragment in the record's own source URL can identify an entire
    // article, as in the standalone Ruby method records.
    if page.source_anchor().as_deref() == Some(&anchor) {
        return Ok(reference(&page.id, None, None));
    }

    if let Some((page, anchor)) = repaired_location(&db, &page, &anchor)? {
        return Ok(reference(&page.id, Some(&anchor), None));
    }
    Ok(Value::Null)
}

fn resolve_sdk_symbol(db: &Connection, pack: &str, version: &str, hash: &str) -> Result<Value> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || !crate::doc_updates::installed_metadata(db)?
            .get(&(pack.to_owned(), version.to_owned()))
            .is_some_and(|metadata| metadata.local_only)
    {
        return Ok(Value::Null);
    }
    let id = format!("{pack}@{version}/symbol/{hash}");
    let exists = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM documents WHERE id=?1 AND pack_id=?2 AND version=?3)",
        params![id, pack, version],
        |row| row.get::<_, bool>(0),
    )?;
    Ok(if exists {
        reference(&id, None, None)
    } else {
        Value::Null
    })
}

fn reference(id: &str, anchor: Option<&str>, focus: Option<&str>) -> Value {
    json!({"id": id, "anchor": anchor, "focus_text": focus})
}

fn stored_html(db: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    db.query_row("SELECT html FROM documents WHERE id=?1", [id], |row| {
        row.get(0)
    })
}

fn has_anchor(html: &str, anchor: &str) -> bool {
    // fetch-docs.py emits escaped text and double-quoted, escaped attributes.
    let escaped = anchor
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    html.split('<').skip(1).any(|tag| {
        let Some((tag, _)) = tag.split_once('>') else {
            return false;
        };
        if !tag.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
            return false;
        }
        let legacy_anchor = tag.starts_with("a ");
        let mut parts = tag.split('"');
        while let (Some(before), Some(value)) = (parts.next(), parts.next()) {
            if value == escaped
                && (before.ends_with(" id=") || legacy_anchor && before.ends_with(" name="))
            {
                return true;
            }
        }
        false
    })
}

fn repaired_location(
    db: &Connection,
    page: &Location,
    anchor: &str,
) -> Result<Option<(Location, String)>> {
    // These aliases were verified against the released corpus. Resolve a
    // redirect only within the original destination's installed revision.
    static REPAIRS: OnceLock<Value> = OnceLock::new();
    let repairs = REPAIRS.get_or_init(|| {
        serde_json::from_str(include_str!("doc_link_repairs.json")).expect("valid link repairs")
    });
    let Ok(source) = Iri::parse(page.source.as_str()) else {
        return Ok(None);
    };
    let base = source.strip_fragment();
    let Some(repair) = repairs[base.as_str()][anchor].as_str() else {
        return Ok(None);
    };
    let destination = IriRef::parse(repair)?.resolve_against(&base)?;
    let destination = destination.borrow();
    let Some(anchor) = fragment(&destination) else {
        return Ok(None);
    };
    let Some(repaired) = locate(db, &destination, &page.pack, &page.version)? else {
        return Ok(None);
    };
    if repaired.pack != page.pack
        || repaired.version != page.version
        || !stored_html(db, &repaired.id)?.is_some_and(|html| has_anchor(&html, &anchor))
    {
        return Ok(None);
    }
    Ok(Some((repaired, anchor)))
}

fn locate(
    db: &Connection,
    target: &Iri<&str>,
    pack: &str,
    version: &str,
) -> Result<Option<Location>> {
    let anchor = fragment(target);
    let base = target.strip_fragment();
    let base = base.as_str();
    let mut candidates = Vec::new();
    let mut lookup = db.prepare_cached(
        "SELECT id,pack_id,version,source_url FROM documents
         WHERE source_url=?1 OR (source_url>=?2 AND source_url<?3)",
    )?;
    // Keep query strings and versioned paths significant. A trailing slash is the only fallback.
    let alternate = if target.query().is_none() {
        Some(
            base.strip_suffix('/')
                .map_or_else(|| format!("{base}/"), str::to_owned),
        )
    } else {
        None
    };
    for url in std::iter::once(base).chain(alternate.as_deref()) {
        let rows = lookup.query_map(params![url, format!("{url}#"), format!("{url}$")], |row| {
            Ok(Location {
                id: row.get(0)?,
                pack: row.get(1)?,
                version: row.get(2)?,
                source: row.get(3)?,
            })
        })?;
        for row in rows {
            candidates.push(row?);
        }
    }
    // A page opened from an older release must never jump to another release of that pack.
    if !pack.is_empty() && !version.is_empty() {
        if candidates.iter().any(|entry| entry.pack == pack)
            && !candidates
                .iter()
                .any(|entry| entry.pack == pack && entry.version == version)
        {
            return Ok(None);
        }
        candidates.retain(|entry| entry.pack != pack || entry.version == version);
    }
    if candidates.iter().any(|entry| entry.pack == pack) {
        candidates.retain(|entry| entry.pack == pack);
    }
    if candidates.iter().any(|entry| {
        entry.source == base
            || entry
                .source
                .strip_prefix(base)
                .is_some_and(|suffix| suffix.starts_with('#'))
    }) {
        candidates.retain(|entry| {
            entry.source == base
                || entry
                    .source
                    .strip_prefix(base)
                    .is_some_and(|suffix| suffix.starts_with('#'))
        });
    }
    let exact_fragment = anchor.is_some()
        && candidates
            .iter()
            .any(|entry| entry.source_anchor() == anchor);
    let wanted_fragment = if exact_fragment {
        anchor.as_deref()
    } else {
        None
    };
    candidates.retain(|entry| entry.source_anchor().as_deref() == wanted_fragment);
    let Some(first) = candidates.first() else {
        return Ok(None);
    };
    let upstream = doc_release(&first.version).0;
    if candidates
        .iter()
        .any(|entry| entry.pack != first.pack || doc_release(&entry.version).0 != upstream)
    {
        return Ok(None);
    }
    candidates.sort_by(|a, b| {
        compare_doc_releases(&b.version, &a.version)
            .then((a.id != related_root(&a.id)).cmp(&(b.id != related_root(&b.id))))
            .then(a.id.cmp(&b.id))
    });
    let first = &candidates[0];
    if candidates.iter().any(|entry| {
        entry.version == first.version && related_root(&entry.id) != related_root(&first.id)
    }) {
        return Ok(None);
    }

    Ok(candidates.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{doc_updates, templates::Stage, Catalog, DocPack};

    #[test]
    fn sdk_links_require_an_installed_local_origin_and_stay_in_its_revision() -> Result<()> {
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
        let db = ctx.database()?;
        let hash = "a".repeat(64);
        let target = format!("swiftui@27.2/symbol/{hash}");
        for (version, local_only) in [("27.2", true), ("27.1", true), ("27.0", false)] {
            let pack: DocPack = serde_json::from_value(json!({
                "id":"swiftui","name":"SwiftUI","stack":"swift","version":version,
                "description":"SDK fixture","source_url":"https://developer.apple.com/documentation/swiftui",
                "license":"Fixture","path":format!("docs/swiftui/{version}"),"local_only":local_only,
            }))?;
            db.execute(
                "INSERT INTO packs VALUES('swiftui',?1,'digest',1)",
                [version],
            )?;
            doc_updates::record_installed(&db, &pack)?;
            db.execute(
                "INSERT INTO documents(id,pack_id,version,source_url) VALUES(?1,'swiftui',?2,?3)",
                params![
                    format!("swiftui@{version}/origin"),
                    version,
                    pack.source_url
                ],
            )?;
            if version != "27.1" {
                db.execute(
                    "INSERT INTO documents(id,pack_id,version,source_url) VALUES(?1,'swiftui',?2,?3)",
                    params![format!("swiftui@{version}/symbol/{hash}"), version, pack.source_url],
                )?;
            }
        }
        let url = format!("{SDK_LINK_PREFIX}{hash}");
        assert_eq!(
            resolve(&ctx, &json!({"url":url,"id":"swiftui@27.2/origin"}))?,
            reference(&target, None, None)
        );
        for request in [
            json!({"url":url,"id":"swiftui@27.1/origin"}),
            json!({"url":url,"id":"swiftui@27.0/origin"}),
            json!({"url":url,"id":"swiftui@27.1/origin","version":"27.2"}),
            json!({"url":url,"pack_id":"swiftui","version":"27.2"}),
            json!({"url":format!("{SDK_LINK_PREFIX}{}", hash.to_uppercase()),"id":"swiftui@27.2/origin"}),
        ] {
            assert_eq!(resolve(&ctx, &request)?, Value::Null, "{request}");
        }
        Ok(())
    }
}
