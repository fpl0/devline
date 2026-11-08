use crate::{
    doc_search::{focus_for_link, related_root},
    markdown::web_url,
    project, text_arg,
    updates::{compare_doc_releases, doc_release},
    Context, DocPack,
};
use anyhow::{bail, Result};
use fluent_uri::{Iri, IriRef};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path, sync::OnceLock};

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
    if !valid_input(raw) {
        return Ok(Value::Null);
    }
    let db = ctx.database()?;
    let origin = origin(&db, request)?;
    let requested_origin = request["id"].as_str().is_some();
    let resolution = if requested_origin && origin.is_none() {
        LinkResolution::Unavailable
    } else {
        resolve_link(&db, request, raw, origin.as_ref(), None, false)?
    };
    Ok(match resolution {
        LinkResolution::Installed(value) => value,
        _ => Value::Null,
    })
}

pub(super) fn status(ctx: &Context, request: &Value) -> Result<Value> {
    text_arg(request, "id")?;
    let Some(urls) = request["urls"].as_array() else {
        bail!("Link status requires a list of URLs");
    };
    if urls.len() > 256 || urls.iter().any(|url| !url.is_string()) {
        bail!("Link status accepts at most 256 URL strings");
    }
    let db = ctx.database()?;
    let Some(origin) = origin(&db, request)? else {
        return Ok(json!([]));
    };
    let releases = request["project_path"]
        .as_str()
        .map(|path| project::link_releases(ctx, &db, Path::new(path)))
        .transpose()?
        .flatten();
    let mut results = Vec::with_capacity(urls.len());
    for url in urls.iter().filter_map(Value::as_str) {
        let raw = url.trim();
        let mut value =
            match resolve_link(&db, request, raw, Some(&origin), releases.as_ref(), true)? {
                LinkResolution::Installed(mut value) => {
                    value["kind"] = json!("installed");
                    value
                }
                LinkResolution::MissingSection => json!({"kind":"missingSection"}),
                LinkResolution::Online(target) => {
                    missing_or_online(ctx, &db, &origin, &target, releases.as_ref())?
                }
                LinkResolution::Unavailable => json!({"kind":"unavailable"}),
            };
        value["url"] = json!(url);
        results.push(value);
    }
    Ok(json!(results))
}

fn missing_or_online(
    ctx: &Context,
    db: &Connection,
    origin: &Location,
    target: &str,
    releases: Option<&BTreeMap<String, String>>,
) -> Result<Value> {
    let candidates = Iri::parse(target)
        .ok()
        .map(|url| candidate_locations(db, &url))
        .transpose()?
        .unwrap_or_default();
    let projected = releases.and_then(|releases| {
        let mut matches = candidates.iter().filter_map(|page| {
            releases
                .get(&page.pack)
                .map(|version| (&page.pack, version))
        });
        let first = matches.next()?;
        matches.all(|candidate| candidate == first).then_some(first)
    });
    let expected = if candidates.iter().any(|page| page.pack == origin.pack) {
        Some((&origin.pack, &origin.version))
    } else {
        projected
    };
    let (pack_id, version) = if let Some((id, version)) = expected {
        (id, Some(version))
    } else if let Some(pack) = known_pack(&ctx.catalog.docs, target, &origin.pack) {
        (
            &pack.id,
            releases.and_then(|releases| releases.get(&pack.id)),
        )
    } else {
        return Ok(json!({"kind":"online"}));
    };
    let present: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM packs WHERE id=?1 AND (?2 IS NULL OR version=?2))",
        params![pack_id, version],
        |row| row.get(0),
    )?;
    let installable = !present
        && ctx.catalog.docs.iter().any(|pack| {
            &pack.id == pack_id && version.is_none_or(|version| &pack.version == version)
        });
    Ok(
        json!({"kind":"missingDocument", "pack_id":pack_id, "version":version, "can_install":installable}),
    )
}

enum LinkResolution {
    Installed(Value),
    MissingSection,
    Online(String),
    Unavailable,
}

fn valid_input(raw: &str) -> bool {
    !raw.is_empty() && raw.len() <= 8192 && !raw.starts_with("//")
}

fn origin(db: &Connection, request: &Value) -> Result<Option<Location>> {
    let Some(id) = request["id"].as_str() else {
        return Ok(None);
    };
    let pack = request["pack_id"].as_str().unwrap_or_default();
    let version = request["version"].as_str().unwrap_or_default();
    let row = db
        .query_row(
            "SELECT pack_id,version,source_url FROM documents WHERE id=?1",
            [id],
            |row| {
                Ok(Location {
                    id: id.into(),
                    pack: row.get(0)?,
                    version: row.get(1)?,
                    source: row.get(2)?,
                })
            },
        )
        .optional()?;
    Ok(row.filter(|row| {
        (pack.is_empty() || pack == row.pack) && (version.is_empty() || version == row.version)
    }))
}

fn resolve_link(
    db: &Connection,
    request: &Value,
    raw: &str,
    origin: Option<&Location>,
    releases: Option<&BTreeMap<String, String>>,
    reader: bool,
) -> Result<LinkResolution> {
    use LinkResolution::*;
    if !valid_input(raw) {
        return Ok(Unavailable);
    }
    let mut pack = request["pack_id"].as_str().unwrap_or_default();
    let mut version = request["version"].as_str().unwrap_or_default();
    if let Some(origin) = origin {
        pack = &origin.pack;
        version = &origin.version;
    }
    if let Some(hash) = raw.strip_prefix(SDK_LINK_PREFIX) {
        if origin.is_some() && pack == crate::apple_docs::PACK_ID {
            let value = resolve_sdk_symbol(db, pack, version, hash)?;
            if !value.is_null() {
                return Ok(Installed(value));
            }
        }
        return Ok(Unavailable);
    }

    let target = if let Ok(url) = Iri::parse(raw) {
        url.normalize()
    } else {
        let Some(source) = origin
            .and_then(|row| Iri::parse(row.source.as_str()).ok())
            .filter(web_url)
        else {
            return Ok(Unavailable);
        };
        let Ok(link) = IriRef::parse(raw) else {
            return Ok(Unavailable);
        };
        let Ok(url) = link.resolve_against(&source.strip_fragment()) else {
            return Ok(Unavailable);
        };
        url.normalize()
    };
    let target = target.borrow();
    if !web_url(&target) {
        return Ok(Unavailable);
    }
    let anchor = fragment(&target);
    if target
        .fragment()
        .is_some_and(|part| !part.as_str().is_empty())
        && anchor.is_none()
    {
        return Ok(Unavailable);
    }
    if reader && raw.starts_with('#') {
        if let (Some(origin), Some(anchor)) = (origin, &anchor) {
            if stored_html(db, &origin.id)?.is_some_and(|html| has_anchor(&html, anchor)) {
                return Ok(Installed(reference(&origin.id, Some(anchor), None)));
            }
        }
    }
    let Some(page) = locate(db, &target, pack, version, releases, reader)? else {
        return Ok(Online(target.as_str().to_owned()));
    };
    let Some(anchor) = anchor else {
        return Ok(Installed(reference(&page.id, None, None)));
    };
    let Some(html) = stored_html(db, &page.id)? else {
        let (id, focus) = focus_for_link(db, &page.id, &anchor)?;
        if reader && focus.is_none() {
            return Ok(if page.source_anchor().as_deref() == Some(&anchor) {
                Installed(reference(&page.id, None, None))
            } else {
                MissingSection
            });
        }
        return Ok(Installed(reference(&id, Some(&anchor), focus.as_deref())));
    };
    if has_anchor(&html, &anchor) {
        return Ok(Installed(reference(&page.id, Some(&anchor), None)));
    }
    // A fragment in the record's own source URL can identify an entire
    // article, as in the standalone Ruby method records.
    if page.source_anchor().as_deref() == Some(&anchor) {
        return Ok(Installed(reference(&page.id, None, None)));
    }

    if let Some((page, anchor)) = repaired_location(db, &page, &anchor)? {
        return Ok(Installed(reference(&page.id, Some(&anchor), None)));
    }
    Ok(MissingSection)
}

fn known_pack<'a>(packs: &'a [DocPack], target: &str, origin: &str) -> Option<&'a DocPack> {
    packs
        .iter()
        .filter_map(|pack| {
            let source = Iri::parse(pack.source_url.as_str()).ok()?.normalize();
            let base = source.strip_fragment();
            let root = base.path().as_str().trim_matches('/').is_empty();
            let base = base.as_str().trim_end_matches('/');
            let suffix = target.strip_prefix(base)?;
            let segment = suffix
                .trim_start_matches('/')
                .split(['/', '#', '?'])
                .next()
                .unwrap_or_default();
            let docs_path = matches!(
                segment,
                "docs"
                    | "api"
                    | "guide"
                    | "guides"
                    | "tutorial"
                    | "tutorials"
                    | "reference"
                    | "learn"
            );
            (suffix.is_empty() || suffix.starts_with(['/', '#', '?']) && (!root || docs_path))
                .then_some(((base.len(), pack.id == origin), pack))
        })
        .max_by_key(|(rank, _)| *rank)
        .map(|(_, pack)| pack)
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
    let Some(repaired) = locate(db, &destination, &page.pack, &page.version, None, false)? else {
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

fn candidate_locations(db: &Connection, target: &Iri<&str>) -> Result<Vec<Location>> {
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
    Ok(candidates)
}

fn locate(
    db: &Connection,
    target: &Iri<&str>,
    pack: &str,
    version: &str,
    releases: Option<&BTreeMap<String, String>>,
    reader: bool,
) -> Result<Option<Location>> {
    let anchor = fragment(target);
    let base = target.strip_fragment();
    let base = base.as_str();
    let mut candidates = candidate_locations(db, target)?;
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
    } else if let Some(releases) = releases.filter(|releases| {
        candidates
            .iter()
            .any(|page| releases.contains_key(&page.pack))
    }) {
        candidates.retain(|entry| releases.get(&entry.pack) == Some(&entry.version));
    } else if reader
        && candidates
            .iter()
            .any(|entry| !crate::doc_library::is_devdocs(&entry.pack))
    {
        candidates.retain(|entry| !crate::doc_library::is_devdocs(&entry.pack));
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
