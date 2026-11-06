use crate::{
    doc_search::{focus_for_link, related_root},
    markdown::web_url,
    text_arg,
    updates::{compare_doc_releases, doc_release},
    Context,
};
use anyhow::Result;
use fluent_uri::{Iri, IriRef};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

struct Location {
    id: String,
    pack: String,
    version: String,
    source: String,
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
            return Ok(Value::Null);
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
    let source_anchor = |entry: &Location| {
        Iri::parse(entry.source.as_str())
            .ok()
            .as_ref()
            .and_then(fragment)
    };
    let exact_fragment = anchor.is_some()
        && candidates
            .iter()
            .any(|entry| source_anchor(entry) == anchor);
    let wanted_fragment = if exact_fragment {
        anchor.as_deref()
    } else {
        None
    };
    candidates.retain(|entry| source_anchor(entry).as_deref() == wanted_fragment);
    let Some(first) = candidates.first() else {
        return Ok(Value::Null);
    };
    let upstream = doc_release(&first.version).0;
    if candidates
        .iter()
        .any(|entry| entry.pack != first.pack || doc_release(&entry.version).0 != upstream)
    {
        return Ok(Value::Null);
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
        return Ok(Value::Null);
    }

    let mut id = first.id.clone();
    let mut focus = None;
    if let Some(anchor) = &anchor {
        let stored_html: bool = db.query_row(
            "SELECT html IS NOT NULL FROM documents WHERE id=?1",
            [&id],
            |row| row.get(0),
        )?;
        if !stored_html {
            (id, focus) = focus_for_link(&db, &id, anchor)?;
        }
    }
    Ok(json!({"id": id, "anchor": anchor, "focus_text": focus}))
}
