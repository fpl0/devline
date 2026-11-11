mod ranking;

use crate::{
    doc_index, doc_updates, markdown, project, text_arg, updates::compare_doc_releases, Context,
};
use anyhow::{bail, Context as _, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::{
    collections::{btree_map::Entry as Slot, BTreeMap, HashSet},
    path::Path,
};

/// The cheap part of an article row: enough to rank it without its text.
#[derive(Clone)]
struct Lead {
    rowid: i64,
    id: String,
    pack_id: String,
    title: String,
}

struct DocScope<'a> {
    term: &'a str,
    releases: String,
    category: &'static str,
    /// Releases and articles in scope.
    packs: usize,
    documents: i64,
    /// Contiguous runs of article rowids, normally one per release.
    spans: Vec<(i64, i64)>,
}

impl<'a> DocScope<'a> {
    fn new(
        ctx: &Context,
        db: &Connection,
        request: &'a Value,
        input: &'a str,
        latest: bool,
    ) -> Result<Self> {
        let needs_stack = input.contains(':')
            || match request["stack"].as_str() {
                Some(stack) => !matches!(stack, "" | "all"),
                None => request["project_stack"]
                    .as_str()
                    .is_some_and(|stack| !stack.is_empty()),
            };
        // The stack of a release is recorded with its metadata. Legacy packs
        // store it only in the article rows, which are costly to read, so
        // neither is read when the request does not use a stack.
        let recorded = if db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='doc_pack_metadata' AND type='table')",
            [],
            |row| row.get::<_, bool>(0),
        )? {
            "(SELECT json_extract(m.json,'$.stack') FROM doc_pack_metadata m
              WHERE m.id=p.id AND m.version=p.version)"
        } else {
            "NULL"
        };
        let mut stmt = db.prepare_cached(&format!(
            "SELECT p.id,p.version,CASE WHEN ?1 THEN coalesce({recorded},(
                 SELECT stack FROM documents d WHERE d.pack_id=p.id AND d.version=p.version LIMIT 1
             )) ELSE '' END
             FROM packs p"
        ))?;
        let installed = stmt
            .query_map([needs_stack], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let known_packs = installed
            .iter()
            .map(|(id, _, _)| id.as_str())
            .chain(ctx.catalog.docs.iter().map(|pack| pack.id.as_str()))
            .collect::<HashSet<_>>();
        let known_stacks = installed
            .iter()
            .map(|(_, _, stack)| stack.as_str())
            .chain(ctx.catalog.docs.iter().map(|pack| pack.stack.as_str()))
            .collect::<HashSet<_>>();
        let (term, selectors) = scoped_term(input, &known_packs, &known_stacks);
        let pack = request["pack_id"].as_str().unwrap_or("");
        let version = request["version"].as_str().unwrap_or("");
        let kind = request["kind"].as_str();
        let automatic = latest
            && (selectors.is_empty() || kind == Some("project"))
            && pack.is_empty()
            && version.is_empty()
            && request["stack"].as_str().is_none();
        let project_releases = if automatic {
            request["project_path"]
                .as_str()
                .map(|path| project::search_releases(ctx, db, Path::new(path)))
                .transpose()?
                .flatten()
        } else {
            None
        };
        let stack = match request["stack"].as_str() {
            Some("all" | "") => "",
            Some(stack) => stack,
            None if selectors.is_empty() && project_releases.is_none() => {
                request["project_stack"].as_str().unwrap_or("")
            }
            None => "",
        };
        let eligible = |id: &str, release: &str, family: &str| {
            let imported = crate::doc_library::is_devdocs(id);
            (match kind {
                Some("installed") => !imported,
                Some("imported") => imported,
                Some("project") => project_releases.is_some(),
                _ => true,
            }) && (stack.is_empty() || stack == family)
                && (pack.is_empty() || pack == id)
                && (version.is_empty() || version == release)
                && project_releases.as_ref().is_none_or(|releases| {
                    releases.get(id).is_some_and(|selected| selected == release)
                })
                && (selectors.is_empty()
                    || selectors
                        .iter()
                        .any(|selector| selector.matches(id, release, family)))
        };
        let mut head = BTreeMap::<&str, &str>::new();
        if latest {
            let preferred = if version.is_empty() && project_releases.is_none() {
                doc_updates::preferences(db)?
            } else {
                BTreeMap::new()
            };
            for (id, release, family) in &installed {
                if eligible(id, release, family) {
                    let current = head.entry(id).or_insert(release);
                    let pin = preferred.get(id).map(String::as_str);
                    if pin == Some(release)
                        || (pin != Some(*current) && compare_doc_releases(release, current).is_gt())
                    {
                        *current = release;
                    }
                }
            }
        }
        let releases = installed
            .iter()
            .filter(|(id, release, family)| {
                eligible(id, release, family)
                    && (!latest
                        || !version.is_empty()
                        || head.get(id.as_str()) == Some(&release.as_str())
                        || selectors.iter().any(|selector| {
                            selector.version.is_some() && selector.matches(id, release, family)
                        }))
            })
            .map(|(id, release, _)| format!("{id}@{release}"))
            .collect::<Vec<_>>();
        let category = match request["category"].as_str().unwrap_or_default() {
            "" | "all" => "",
            "guide" => "guide",
            "api" => "api",
            "compatibility" => "compatibility",
            "reference" => "reference",
            _ => bail!("Choose guide, api, compatibility, reference, or all for category"),
        };
        let mut scope = Self {
            term,
            packs: releases.len(),
            releases: serde_json::to_string(&releases)?,
            category,
            documents: 0,
            spans: Vec::new(),
        };
        scope.materialize(db)?;
        Ok(scope)
    }

    /// Stores the scope as ranges of article rowids. A release is imported in
    /// one transaction, so its articles are normally one contiguous range, and
    /// every later filter is an integer comparison instead of an article read.
    fn materialize(&mut self, db: &Connection) -> Result<()> {
        db.execute_batch(
            "PRAGMA temp_store=MEMORY;
             CREATE TEMP TABLE IF NOT EXISTS doc_scope(lo INTEGER PRIMARY KEY, hi INTEGER NOT NULL);
             DELETE FROM temp.doc_scope;",
        )?;
        let mut ranges = Vec::<(i64, i64)>::new();
        let mut scattered = Vec::new();
        {
            let mut stmt = db.prepare_cached(
                "SELECT p.id,p.version,p.count,
                    (SELECT d.rowid FROM documents d INDEXED BY documents_pack
                     WHERE d.pack_id=p.id AND d.version=p.version ORDER BY d.rowid LIMIT 1),
                    (SELECT d.rowid FROM documents d INDEXED BY documents_pack
                     WHERE d.pack_id=p.id AND d.version=p.version ORDER BY d.rowid DESC LIMIT 1)
                 FROM json_each(?1) r JOIN packs p
                 ON p.id=substr(r.value,1,instr(r.value,'@')-1)
                 AND p.version=substr(r.value,instr(r.value,'@')+1)",
            )?;
            let mut rows = stmt.query([&self.releases])?;
            while let Some(row) = rows.next()? {
                let (Some(low), Some(high)) =
                    (row.get::<_, Option<i64>>(3)?, row.get::<_, Option<i64>>(4)?)
                else {
                    continue;
                };
                if self.category.is_empty() && row.get::<_, Option<i64>>(2)? == Some(high - low + 1)
                {
                    ranges.push((low, high));
                } else {
                    scattered.push((row.get::<_, String>(0)?, row.get::<_, String>(1)?));
                }
            }
        }
        // A category, or a release whose rows are not contiguous, is read from
        // the covering index and folded into ranges.
        let filter = if self.category.is_empty() {
            "documents d INDEXED BY documents_pack WHERE d.pack_id=?1 AND d.version=?2 AND ?3=''"
        } else {
            "documents d WHERE d.pack_id=?1 AND d.version=?2 AND d.category=?3"
        };
        let mut stmt =
            db.prepare_cached(&format!("SELECT d.rowid FROM {filter} ORDER BY d.rowid"))?;
        for (pack, version) in scattered {
            let mut rows = stmt.query(params![pack, version, self.category])?;
            while let Some(row) = rows.next()? {
                let rowid: i64 = row.get(0)?;
                match ranges.last_mut() {
                    Some((_, high)) if *high + 1 == rowid => *high = rowid,
                    _ => ranges.push((rowid, rowid)),
                }
            }
        }
        ranges.sort_unstable();
        self.spans.clone_from(&ranges);
        let mut insert = db.prepare_cached("INSERT INTO temp.doc_scope(lo,hi) VALUES(?1,?2)")?;
        let mut merged: Option<(i64, i64)> = None;
        for (low, high) in ranges {
            self.documents += high - low + 1;
            match &mut merged {
                Some((_, end)) if *end + 1 >= low => *end = (*end).max(high),
                _ => {
                    if let Some((start, end)) = merged.replace((low, high)) {
                        insert.execute(params![start, end])?;
                    }
                }
            }
        }
        if let Some((start, end)) = merged {
            insert.execute(params![start, end])?;
        }
        Ok(())
    }

    /// Rowid windows that hold a narrow scope, to confine a full-text query
    /// to. A query confined to a window skips every match outside it, which
    /// matters when a common word or prefix matches most of the library.
    fn article_windows(&self) -> Vec<(i64, i64)> {
        let mut windows = Vec::<(i64, i64)>::new();
        for &(low, high) in &self.spans {
            match windows.last_mut() {
                Some((_, end)) if *end + 1 >= low => *end = (*end).max(high),
                _ => windows.push((low, high)),
            }
        }
        if self.documents <= NARROW_SCOPE && windows.len() <= MOST_WINDOWS {
            windows
        } else {
            vec![(i64::MIN, i64::MAX)]
        }
    }

    /// The same for the section index. A release is indexed in one
    /// transaction, so its sections are one run of rowids, in no particular
    /// order within the run. The bounds are read from the covering index,
    /// which is quick for a small scope, and rows are still checked against
    /// the scope.
    fn section_windows(&self, db: &Connection) -> Result<Vec<(i64, i64)>> {
        let everything = vec![(i64::MIN, i64::MAX)];
        if self.spans.is_empty() || self.spans.len() > 64 || self.documents > SMALL_SCOPE {
            return Ok(everything);
        }
        let mut bounds = db.prepare_cached(
            "SELECT min(s.rowid),max(s.rowid) FROM doc_sections s INDEXED BY doc_sections_document
             WHERE s.document_rowid BETWEEN ?1 AND ?2",
        )?;
        let mut runs = Vec::new();
        for (low, high) in &self.spans {
            let (start, end): (Option<i64>, Option<i64>) =
                bounds.query_row(params![low, high], |row| Ok((row.get(0)?, row.get(1)?)))?;
            if let (Some(start), Some(end)) = (start, end) {
                runs.push((start, end));
            }
        }
        runs.sort_unstable();
        let mut windows = Vec::<(i64, i64)>::new();
        for (start, end) in runs {
            match windows.last_mut() {
                Some((_, last)) if *last + 1 >= start => *last = (*last).max(end),
                _ => windows.push((start, end)),
            }
        }
        Ok(if windows.is_empty() || windows.len() > MOST_WINDOWS {
            everything
        } else {
            windows
        })
    }

    /// SQL that is true when the article with this rowid is in scope.
    fn contains_rowid(&self, rowid: &str) -> String {
        format!(
            "(SELECT hi FROM temp.doc_scope WHERE lo<={rowid} ORDER BY lo DESC LIMIT 1)>={rowid}"
        )
    }

    /// SQL that is true when the article with this ID is in scope. IDs begin
    /// with `pack@version/`, so without a category no other table is read.
    /// The statement binds the scope releases as `?1`.
    fn contains_id(&self, id: &str) -> String {
        if self.category.is_empty() {
            format!("substr({id},1,instr({id},'/')-1) IN (SELECT value FROM json_each(?1))")
        } else {
            format!(
                "EXISTS(SELECT 1 FROM documents scoped WHERE scoped.id={id} AND {})",
                self.contains_rowid("scoped.rowid")
            )
        }
    }
}

struct Selector<'a> {
    id: String,
    version: Option<&'a str>,
    stack: bool,
}

impl Selector<'_> {
    fn matches(&self, id: &str, release: &str, stack: &str) -> bool {
        self.id == if self.stack { stack } else { id }
            && self.version.is_none_or(|version| version == release)
    }
}

fn scoped_term<'a>(
    input: &'a str,
    packs: &HashSet<&str>,
    stacks: &HashSet<&str>,
) -> (&'a str, Vec<Selector<'a>>) {
    let input = input.trim();
    let Some((prefix, term)) = input.split_once(':') else {
        return (input, vec![]);
    };
    if term.starts_with([':', '/']) {
        return (input, vec![]);
    }
    let mut selectors = Vec::new();
    for scope in prefix.split(',') {
        let (name, version) = scope
            .trim()
            .split_once('@')
            .map_or((scope.trim(), None), |(name, version)| {
                (name, Some(version))
            });
        let name = name.to_ascii_lowercase();
        let id = match name.as_str() {
            "py" => "python",
            "cs" | "c#" => "csharp",
            ".net" => "dotnet",
            "rb" => "ruby",
            "rs" => "rust",
            "ex" => "elixir",
            "golang" => "go",
            _ => &name,
        };
        let stack = !packs.contains(id) && stacks.contains(id);
        if (!stack && !packs.contains(id))
            || selectors.len() == 8
            || version.is_some_and(|version| {
                stack
                    || version.is_empty()
                    || !version
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b".-+_".contains(&byte))
            })
        {
            return (input, vec![]);
        }
        selectors.push(Selector {
            id: id.into(),
            version,
            stack,
        });
    }
    (term.trim(), selectors)
}

const ALLOWED_RELEASES: &str = "(d.pack_id||'@'||d.version) IN (SELECT value FROM json_each(?1))";

struct DocSummary {
    id: String,
    pack_id: String,
    title: String,
    stack: String,
    version: String,
    source_url: String,
    category: String,
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
        category: row.get(6)?,
        snippet: row.get(7)?,
    })
}

impl DocSummary {
    fn json(self, ctx: &Context, anchor: Option<String>, symbol: Option<String>) -> Value {
        let title = self.display_title(ctx);
        let mut value = json!({
            "id": self.id,
            "pack_id": self.pack_id,
            "title": title,
            "stack": self.stack,
            "version": self.version,
            "source_url": self.source_url,
            "snippet": self.snippet,
            "category": self.category,
            "match_kind": if symbol.is_some() { "symbol" } else { "article" },
        });
        if let Some(anchor) = anchor {
            value["match_text"] = json!(anchor);
        }
        if let Some(symbol) = symbol {
            value["symbol"] = json!(symbol);
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

pub(super) fn plain_title(title: &str, fallback: &str) -> String {
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

const DOC_COLUMNS: &str = "d.id,d.pack_id,d.title,d.stack,d.version,d.source_url,d.category";
const DOC_PREVIEW: &str = "substr(d.body,1,200)";

pub(super) fn navigation(ctx: &Context, request: &Value) -> Result<Value> {
    let pack = text_arg(request, "pack_id")?;
    let version = text_arg(request, "version")?;
    let db = ctx.database()?;
    if crate::doc_library::validate_doc_identity(pack, version).is_err() {
        return Ok(json!({"groups": [], "has_ungrouped": false}));
    }
    // IDs begin with `pack@version/`. `0` is the next byte after `/`, so
    // both queries can read the release from the existing primary indexes.
    let start = format!("{pack}@{version}/");
    let end = format!("{pack}@{version}0");
    let mut statement = db.prepare(
        "SELECT group_name FROM (
             SELECT DISTINCT group_name FROM doc_navigation
             WHERE document_id>=?1 AND document_id<?2 AND document_id NOT GLOB '*::part-[0-9]*'
             UNION ALL
             SELECT NULL WHERE EXISTS (
                 SELECT 1 FROM documents d
                 WHERE d.id>=?1 AND d.id<?2 AND d.id NOT GLOB '*::part-[0-9]*'
                 AND NOT EXISTS (SELECT 1 FROM doc_navigation n WHERE n.document_id=d.id)
             )
         ) ORDER BY group_name COLLATE NOCASE,group_name",
    )?;
    let groups = statement
        .query_map(params![start, end], |row| row.get::<_, Option<String>>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(
        json!({"groups": groups.iter().flatten().collect::<Vec<_>>(), "has_ungrouped": groups.contains(&None)}),
    )
}

pub(super) fn related_root(id: &str) -> &str {
    id.rsplit_once("::part-")
        .filter(|(_, number)| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
        .map_or(id, |(root, _)| root)
}

fn symbol_pieces(alias: &str) -> Option<(&str, &str)> {
    let reference = doc_index::without_call(alias);
    let reference = reference
        .rsplit_once('/')
        .filter(|(_, arity)| !arity.is_empty() && arity.bytes().all(|b| b.is_ascii_digit()))
        .map_or(alias, |(name, _)| name);
    let mut parts = reference
        .rsplit(['.', ':', '#'])
        .filter(|part| !part.is_empty());
    let symbol = parts.next()?;
    let owner = parts.next().unwrap_or("");
    if reference.contains(char::is_whitespace) || !doc_index::symbol_component(symbol) {
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
    let query = doc_index::without_call(query).to_lowercase();
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
    mut original: DocSummary,
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
        let body: String = row.get(7)?;
        if original.snippet.is_empty() {
            original.snippet = compact_excerpt(&body);
        }
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

struct AliasMatch {
    hit: DocSummary,
    label: String,
    tier: u8,
    depth: u32,
    /// The article is named after the symbol rather than merely containing it.
    named: bool,
    /// Letters the query still lacks when this is a completion.
    unfinished: u32,
    /// Results in one bucket are equally good answers and docsets take turns.
    bucket: u8,
}

/// One indexed name for an article, before the article row is read.
struct SymbolRow {
    id: String,
    label: String,
    tier: u8,
    depth: u32,
    unfinished: u32,
}

fn api_priority(id: &str) -> u8 {
    if ["/api/", "/std/", "/method/", "/fn.", "/struct.", "/trait."]
        .iter()
        .any(|part| id.contains(part))
    {
        0
    } else if id.ends_with("/index.md") {
        2
    } else {
        1
    }
}

fn pack_of(id: &str) -> &str {
    id.split_once('@').map_or(id, |(pack, _)| pack)
}

/// Publisher references lead personal imports, and among imports the web
/// platform and the most widely used languages and tools lead the long tail.
fn pack_prior(pack: &str) -> u8 {
    match pack.strip_prefix("devdocs-") {
        None => 2,
        Some(
            "bash" | "c" | "cpp" | "css" | "docker" | "dom" | "git" | "go" | "html" | "http"
            | "javascript" | "kubernetes" | "node" | "openjdk" | "php" | "postgresql" | "python"
            | "react" | "ruby" | "rust" | "sqlite" | "typescript",
        ) => 1,
        Some(_) => 0,
    }
}

fn prefix_end(query: &str) -> String {
    query
        .char_indices()
        .rev()
        .find_map(|(offset, ch)| {
            let next = u32::from(ch) + 1;
            char::from_u32(if next == 0xd800 { 0xe000 } else { next })
                .map(|next| format!("{}{next}", &query[..offset]))
        })
        .unwrap_or_default()
}

/// Index rows read for one lookup. A common prefix can cover a large part of
/// the index, so completions come from the first names in order.
const SCANNED_NAMES: i64 = 20_000;
/// Below this many articles, completions are read per article instead, which
/// finds every completion however many other docsets share the prefix.
const NARROW_SCOPE: i64 = 4_000;
/// A narrow scope in more pieces than this is queried as a whole.
const MOST_WINDOWS: usize = 4;
/// Below this many articles, the bounds of a scope's sections are worth reading.
const SMALL_SCOPE: i64 = 1_500;

fn symbol_rows(
    db: &Connection,
    scope: &DocScope,
    query: &str,
    prefix: bool,
) -> Result<Vec<SymbolRow>> {
    let mut rows = Vec::new();
    let length = query.chars().count() as u32;
    let upper = prefix_end(query);
    let in_scope = scope.contains_id("a.document_id");
    {
        let mut stmt = db.prepare_cached(&format!(
            "SELECT a.document_id,a.alias FROM aliases a INDEXED BY aliases_lookup
             WHERE a.alias=?2 AND {in_scope} LIMIT {SCANNED_NAMES}"
        ))?;
        let mut found = stmt.query(params![scope.releases, query])?;
        while let Some(row) = found.next()? {
            rows.push(SymbolRow {
                id: row.get(0)?,
                label: row.get(1)?,
                tier: 0,
                depth: u32::MAX,
                unfinished: 0,
            });
        }
    }
    if prefix && !upper.is_empty() {
        // Bounds are bound parameters, so punctuation in names is literal.
        let mut stmt = db.prepare_cached(&format!(
            "SELECT a.document_id,a.alias FROM (
                 SELECT document_id,alias FROM aliases INDEXED BY aliases_lookup
                 WHERE alias>?2 AND alias<?3 ORDER BY alias LIMIT {SCANNED_NAMES}
             ) a WHERE {in_scope}"
        ))?;
        let mut found = stmt.query(params![scope.releases, query, upper])?;
        while let Some(row) = found.next()? {
            let label: String = row.get(1)?;
            rows.push(SymbolRow {
                id: row.get(0)?,
                unfinished: (label.chars().count() as u32).saturating_sub(length),
                label,
                tier: 4,
                depth: u32::MAX,
            });
        }
    }
    let Some(key) = doc_index::symbol_key(query) else {
        return Ok(rows);
    };
    let in_scope = scope.contains_id("s.document_id");
    {
        let mut stmt = db.prepare_cached(&format!(
            "SELECT s.document_id,s.alias,s.depth FROM doc_symbol_keys s
             WHERE s.key=?2 AND {in_scope} LIMIT {SCANNED_NAMES}"
        ))?;
        let mut found = stmt.query(params![scope.releases, key])?;
        while let Some(row) = found.next()? {
            let id: String = row.get(0)?;
            let label: String = row.get(1)?;
            let depth: u32 = row.get(2)?;
            let tier = if pack_of(&id) != crate::apple_docs::PACK_ID
                && label.eq_ignore_ascii_case(query)
            {
                0
            } else if depth == 0 {
                1
            } else {
                2
            };
            rows.push(SymbolRow {
                id,
                label,
                tier,
                depth,
                unfinished: 0,
            });
        }
    }
    let upper = prefix_end(&key);
    let length = key.chars().count() as u32;
    if prefix && length >= 3 && !upper.is_empty() {
        let sql = if scope.documents <= NARROW_SCOPE {
            "SELECT s.document_id,s.alias,s.depth,s.key FROM temp.doc_scope r
             JOIN documents d ON d.rowid BETWEEN r.lo AND r.hi
             JOIN doc_symbol_keys s INDEXED BY doc_symbol_document
             ON s.document_id=d.id AND s.key>?2 AND s.key<?3"
                .to_owned()
        } else {
            format!(
                "SELECT s.document_id,s.alias,s.depth,s.key FROM (
                     SELECT document_id,alias,depth,key FROM doc_symbol_keys
                     WHERE key>?2 AND key<?3 ORDER BY key LIMIT {SCANNED_NAMES}
                 ) s WHERE {in_scope}"
            )
        };
        let mut stmt = db.prepare_cached(&sql)?;
        let mut found = stmt.query(params![scope.releases, key, upper])?;
        while let Some(row) = found.next()? {
            let matched: String = row.get(3)?;
            rows.push(SymbolRow {
                id: row.get(0)?,
                label: row.get(1)?,
                tier: 4,
                depth: row.get(2)?,
                unfinished: (matched.chars().count() as u32).saturating_sub(length),
            });
        }
    }
    Ok(rows)
}

/// Articles that hold any of the given spellings of a symbol.
fn variant_rows(db: &Connection, scope: &DocScope, variants: &[String]) -> Result<Vec<SymbolRow>> {
    let keys: Vec<_> = variants
        .iter()
        .filter_map(|variant| doc_index::symbol_key(variant))
        .collect();
    if keys.is_empty() {
        return Ok(vec![]);
    }
    let mut stmt = db.prepare_cached(&format!(
        "SELECT s.document_id,s.alias,s.depth FROM json_each(?2) variant
         CROSS JOIN doc_symbol_keys s ON s.key=variant.value
         WHERE {} LIMIT {SCANNED_NAMES}",
        scope.contains_id("s.document_id")
    ))?;
    let rows = stmt
        .query_map(
            params![scope.releases, serde_json::to_string(&keys)?],
            |row| {
                let depth: u32 = row.get(2)?;
                Ok(SymbolRow {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    tier: if depth == 0 { 1 } else { 2 },
                    depth,
                    unfinished: 0,
                })
            },
        )?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

/// Whether an article is about the symbol itself: `HashMap in std::collections`
/// for `std::collections::HashMap`, but not `Enum` for `Enum.map/2`.
fn names_article(title: &str, label: &str) -> bool {
    let title = plain_title(title, "");
    let name = doc_index::without_call(ranking::title_name(&title));
    let label = doc_index::without_call(label);
    let leaf = symbol_pieces(label).map_or(label, |(symbol, _)| symbol);
    // Web references title a member `EventTarget: addEventListener() method`.
    let member = name.split_once(": ").map_or(name, |(_, member)| {
        doc_index::without_call(member.split(' ').next().unwrap_or(member))
    });
    name.eq_ignore_ascii_case(label)
        || name.eq_ignore_ascii_case(leaf)
        || member.eq_ignore_ascii_case(leaf)
}

/// Reorders each run of equally ranked results so that docsets take turns.
/// One docset can hold hundreds of members with a common name.
fn take_turns<T, G: PartialEq>(
    items: Vec<T>,
    group: impl Fn(&T) -> G,
    pack: impl Fn(&T) -> &str,
) -> Vec<T> {
    let mut output = Vec::with_capacity(items.len());
    let mut run: Vec<T> = Vec::new();
    let flush = |run: &mut Vec<T>, output: &mut Vec<T>| {
        let mut turns = BTreeMap::<String, usize>::new();
        let mut keyed: Vec<_> = run
            .drain(..)
            .enumerate()
            .map(|(index, item)| {
                let turn = turns.entry(pack(&item).to_owned()).or_default();
                *turn += 1;
                let prior = std::cmp::Reverse(pack_prior(pack(&item)));
                ((*turn, prior, index), item)
            })
            .collect();
        keyed.sort_by_key(|(key, _)| *key);
        output.extend(keyed.into_iter().map(|(_, item)| item));
    };
    for item in items {
        if run
            .first()
            .is_some_and(|first| group(first) != group(&item))
        {
            flush(&mut run, &mut output);
        }
        run.push(item);
    }
    flush(&mut run, &mut output);
    output
}

fn hydrate(db: &Connection, id: &str) -> Result<Option<DocSummary>> {
    let mut stmt = db.prepare_cached(&format!(
        "SELECT {DOC_COLUMNS},'' FROM documents d WHERE d.id=?1"
    ))?;
    Ok(stmt.query_row([id], summary).optional()?)
}

/// Orders indexed names. For a bare name, articles named after it lead and
/// members that merely carry the name follow, whichever way each docset
/// indexed them. For a qualified name, the closeness of the match leads.
fn rank_symbols(
    db: &Connection,
    scope: &DocScope,
    mut rows: Vec<SymbolRow>,
    bare: bool,
    limit: i64,
) -> Result<Vec<AliasMatch>> {
    rows.sort_by(|a, b| {
        (
            a.tier,
            api_priority(&a.id),
            a.unfinished,
            a.depth,
            a.label.len(),
            &a.id,
        )
            .cmp(&(
                b.tier,
                api_priority(&b.id),
                b.unfinished,
                b.depth,
                b.label.len(),
                &b.id,
            ))
    });
    // One row per article. An alias stored in lower case takes its spelling
    // from the same name indexed as a symbol.
    let mut kept = BTreeMap::<String, usize>::new();
    let mut unique: Vec<SymbolRow> = Vec::with_capacity(rows.len());
    for row in rows {
        match kept.get(related_root(&row.id)) {
            Some(&index) => {
                let first = &mut unique[index];
                if first.id == row.id
                    && first.label != row.label
                    && first.label.eq_ignore_ascii_case(&row.label)
                    && !first.label.chars().any(char::is_uppercase)
                {
                    first.label = row.label;
                }
            }
            None => {
                kept.insert(related_root(&row.id).to_owned(), unique.len());
                unique.push(row);
            }
        }
    }
    let mut rows = unique;
    // Titles decide the final order, so they are read for the leading names only.
    let (mut exact, mut completions) = (0, 0);
    rows.retain(|row| {
        let (count, most) = if row.tier <= 2 {
            (&mut exact, 2_500)
        } else {
            (&mut completions, 400)
        };
        *count += 1;
        *count <= most
    });
    let mut title = db.prepare_cached("SELECT title FROM documents WHERE id=?1")?;
    let mut ranked = Vec::with_capacity(rows.len());
    for row in rows {
        let Some(text) = title
            .query_row([&row.id], |found| found.get::<_, Option<String>>(0))
            .optional()?
        else {
            continue;
        };
        let text = text.unwrap_or_default();
        let named = names_article(&text, &row.label);
        let bucket = match (bare, row.tier <= 2) {
            (true, true) => u8::from(!named),
            (true, false) => 2 + u8::from(!named),
            (false, _) => row.tier * 2 + u8::from(!named),
        };
        ranked.push((row, text, (bucket, named)));
    }
    ranked.sort_by(|(a, a_title, a_bucket), (b, b_title, b_bucket)| {
        (
            a_bucket.0,
            a.tier,
            api_priority(&a.id),
            a.unfinished,
            a.depth,
            a.label.len(),
            a_title,
            &a.id,
        )
            .cmp(&(
                b_bucket.0,
                b.tier,
                api_priority(&b.id),
                b.unfinished,
                b.depth,
                b.label.len(),
                b_title,
                &b.id,
            ))
    });
    if scope.packs > 1 {
        ranked = take_turns(
            ranked,
            |(_, _, (bucket, _))| *bucket,
            |(row, _, _)| pack_of(&row.id),
        );
    }
    ranked.truncate(limit as usize);
    let mut matches = Vec::with_capacity(ranked.len());
    for (row, _, (bucket, named)) in ranked {
        if let Some(hit) = hydrate(db, &row.id)? {
            matches.push(AliasMatch {
                hit,
                label: row.label,
                tier: row.tier,
                depth: row.depth,
                named,
                unfinished: row.unfinished,
                bucket,
            });
        }
    }
    Ok(matches)
}

fn alias_candidates(
    db: &Connection,
    scope: &DocScope,
    query: &str,
    prefix: bool,
    limit: i64,
) -> Result<Vec<AliasMatch>> {
    let bare = !doc_index::symbol_key(query).is_some_and(|key| key.contains('.'));
    rank_symbols(
        db,
        scope,
        symbol_rows(db, scope, query, prefix)?,
        bare,
        limit,
    )
}

fn display_symbol(alias: &str, hit: &DocSummary, anchor: Option<&str>) -> Option<String> {
    doc_index::symbol_key(alias)?;
    let label = doc_index::without_call(alias);
    let qualified = label.contains(['.', '#', ':', '/']);
    let declares =
        anchor.is_some_and(|line| find_focus(line, label).is_some_and(|focus| focus.score >= 85));
    if !qualified && !declares && api_priority(&hit.id) != 0 {
        return None;
    }
    let anchor = anchor.unwrap_or_default();
    // Old imports stored aliases in lowercase. Recover spelling from the visible declaration.
    let lower = anchor.to_lowercase();
    let separators = ['.', '#', ':', '/'];
    let mut spelling = String::with_capacity(label.len());
    for component in label.split_inclusive(separators) {
        let segment = component.trim_end_matches(separators);
        let leaf = component == segment || component.ends_with('/');
        let cased = if segment.is_empty() || segment.bytes().any(|byte| byte.is_ascii_uppercase()) {
            None
        } else {
            // The declaration shows how the name is written: `path.join` in
            // an article titled Path. The title is the fallback.
            lower
                .match_indices(segment)
                .find_map(|(start, _)| {
                    let end = start + segment.len();
                    let word = |ch: char| ch.is_alphanumeric() || ch == '_';
                    let suffix = &lower[end..];
                    (!lower[..start].ends_with(word)
                        && !suffix.starts_with(word)
                        && (leaf || suffix.starts_with(['.', '#', ':'])))
                    .then(|| anchor.get(start..end))
                    .flatten()
                })
                .or_else(|| {
                    hit.title
                        .eq_ignore_ascii_case(segment)
                        .then_some(hit.title.as_str())
                })
        };
        spelling.push_str(cased.unwrap_or(segment));
        spelling.push_str(&component[segment.len()..]);
    }
    Some(spelling)
}

/// A key that sorts titles the way a reader orders them: numbers by value, so
/// that chapter 2 precedes chapter 10, and letters without regard to case.
/// Markup and quoting around a name, as in `<StrictMode>`, is not part of its
/// place in the alphabet.
fn natural_key(title: &str) -> String {
    let trimmed = title.trim_start_matches(|ch: char| !ch.is_alphanumeric());
    let title = if trimmed.is_empty() { title } else { trimmed };
    let mut key = String::with_capacity(title.len() + 4);
    let mut chars = title.chars().peekable();
    while let Some(ch) = chars.next() {
        if !ch.is_ascii_digit() {
            key.extend(ch.to_lowercase());
            continue;
        }
        let mut digits = String::from(ch);
        while let Some(digit) = chars.next_if(char::is_ascii_digit) {
            digits.push(digit);
        }
        // A number with more digits is larger, so its length leads. The
        // length stays below the letters, where digits sort.
        let value = digits.trim_start_matches('0');
        key.push(char::from(b'0' + value.len().min(15) as u8));
        key.push_str(value);
    }
    key
}

fn browse(ctx: &Context, db: &Connection, request: &Value, limit: i64) -> Result<Value> {
    let offset = request["offset"].as_u64().unwrap_or(0).min(100_000) as usize;
    let group = request["group"].as_str();
    let (group_join, group_filter) = if group.is_some() {
        (
            "LEFT JOIN doc_navigation n ON n.document_id=d.id",
            "AND coalesce(n.group_name,'')=?1",
        )
    } else {
        ("", "")
    };
    // Order lightweight keys before fetching article previews. SQLite has no
    // numeric-aware collation, so the titles of the browsed scope are ordered here.
    let sql = format!(
        "SELECT d.rowid,d.id,d.title FROM temp.doc_scope r
         JOIN documents d ON d.rowid BETWEEN r.lo AND r.hi {group_join}
         WHERE d.id NOT GLOB '*::part-[0-9]*' {group_filter}"
    );
    let mut stmt = db.prepare(&sql)?;
    let mut rows = match group {
        Some(group) => stmt.query([group])?,
        None => stmt.query([])?,
    };
    let mut articles = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get_ref(1)?.as_str()?;
        let title = row.get_ref(2)?.as_str_or_null()?.unwrap_or_default();
        // Most titles are plain text. The rest are ordered as they are shown.
        let key = if title.contains('<') || title.trim().is_empty() {
            let pack = pack_of(id);
            let fallback = ctx
                .catalog
                .docs
                .iter()
                .find(|candidate| candidate.id == pack)
                .map_or(pack, |candidate| candidate.name.as_str());
            natural_key(&plain_title(title, fallback))
        } else {
            natural_key(title)
        };
        articles.push((key, id.to_owned(), row.get::<_, i64>(0)?));
    }
    articles.sort_unstable();
    let mut page = db.prepare_cached(&format!(
        "SELECT {DOC_COLUMNS},{DOC_PREVIEW} FROM documents d WHERE d.rowid=?1"
    ))?;
    let mut results = Vec::new();
    for (_, _, rowid) in articles.into_iter().skip(offset).take(limit as usize) {
        results.push(page.query_row([rowid], summary)?.json(ctx, None, None));
    }
    Ok(json!(results))
}

/// Full-text candidates for a query, before their articles are read.
#[derive(Default)]
struct Gathered {
    rows: Vec<ranking::Raw>,
    /// Articles in which the words are adjacent.
    phrase: HashSet<i64>,
    /// Articles found only because their title has one of the words.
    titled: HashSet<i64>,
}

impl Gathered {
    /// Nothing matched the whole query. A title that shares one word does not count.
    fn is_empty(&self) -> bool {
        self.articles() == 0
    }

    /// Articles that matched the whole query.
    fn articles(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.document)
            .filter(|article| !self.titled.contains(article))
            .collect::<HashSet<_>>()
            .len()
    }
}

/// How a query is looked up in the full-text indexes.
#[derive(Clone, Copy)]
struct Lookup {
    /// The last word may be unfinished.
    typing: bool,
    /// The last word is a letter or two, left out of the index query.
    fragment: bool,
    /// The words are looked up together, exactly as written.
    literal: bool,
    /// Results already found by name.
    have: usize,
    window: i64,
    limit: i64,
}

fn gather(
    db: &Connection,
    scope: &DocScope,
    terms: &[Vec<String>],
    lookup: Lookup,
) -> Result<Gathered> {
    use ranking::Completion;
    let Lookup {
        typing,
        fragment,
        literal,
        have,
        window,
        limit,
    } = lookup;
    let mut gathered = Gathered::default();
    let mut articles = HashSet::new();
    if literal {
        // A flag or dotted name is looked up exactly as written.
        let last = terms.last().map_or(0, |choices| choices[0].chars().count());
        let completion = if typing && last >= 4 {
            Completion::Typing
        } else {
            Completion::None
        };
        let terms = if fragment {
            &terms[..terms.len() - 1]
        } else {
            terms
        };
        // Completing the last word of a phrase is affordable among headings.
        let written = ranking::literal_expression(terms, completion);
        gathered.rows = ranking::sections(db, scope, &written, window)?;
        let written = ranking::literal_expression(terms, Completion::None);
        for row in ranking::documents(db, scope, &written, false, window)? {
            gathered.phrase.insert(row.document);
            gathered.rows.push(row);
        }
        return Ok(gathered);
    }
    // A fragment of one or two letters completes too many words to look up.
    // The words before it are looked up and the fragment is left to scoring.
    let required = if fragment {
        &terms[..terms.len() - 1]
    } else {
        terms
    };
    let strict = ranking::expression(required, false, Completion::None);
    let flexible = ranking::expression(required, true, Completion::None);
    let mut found = HashSet::new();
    if required.len() == 1 {
        // Ranking every article or heading that mentions a common word is
        // the costly part of a search. Titles are ranked first, and the rest
        // only when names and titles do not fill the results.
        for row in ranking::documents(db, scope, &flexible, true, window)? {
            if articles.insert(row.document) {
                found.insert(row.document);
                gathered.rows.push(row);
            }
        }
    }
    let filled = found.len() + have >= limit as usize;
    if !(filled && ranking::floods_sections(db, &strict)?) {
        // Headings are short, so the index ranks a signature made of the
        // words above a sentence that contains them. Scoring sees both.
        for row in ranking::sections(db, scope, &strict, window * 3)? {
            found.insert(row.document);
            gathered.rows.push(row);
        }
    }
    if required.len() > 1 || found.len() < limit as usize {
        for row in ranking::documents(db, scope, &flexible, false, window)? {
            if articles.insert(row.document) {
                found.insert(row.document);
                gathered.rows.push(row);
            }
        }
    }
    let last = required
        .last()
        .map_or(0, |choices| choices[0].chars().count());
    if typing && !fragment && last >= 3 {
        // Completing a word costs the same however narrow the scope, so each
        // step runs only while the results are still short. `have` counts the
        // names that already complete the word.
        let few = |found: &HashSet<i64>| found.len() + have < limit as usize;
        let completed = ranking::expression(required, true, Completion::Typing);
        if required.len() == 1 || few(&found) {
            // A title that completes the word is the cheapest and best find.
            for row in ranking::documents(db, scope, &completed, true, limit * 2)? {
                if articles.insert(row.document) {
                    found.insert(row.document);
                    gathered.rows.push(row);
                }
            }
        }
        if few(&found) && (last >= 4 || required.len() > 1) {
            let headings = ranking::expression(required, false, Completion::Typing);
            for row in ranking::sections(db, scope, &headings, window)? {
                found.insert(row.document);
                gathered.rows.push(row);
            }
        }
        if found.is_empty() && have == 0 && last >= 5 {
            for row in ranking::documents(db, scope, &completed, false, window)? {
                if articles.insert(row.document) {
                    gathered.rows.push(row);
                }
            }
        }
    }
    // An article named for one of the words is a candidate even when its text
    // uses other words for the rest: `parse json` and the JSON module. Short
    // words are in too many titles to say what an article is about.
    let subjects: Vec<_> = required
        .iter()
        .filter(|choices| choices[0].chars().count() >= 4)
        .cloned()
        .collect();
    if (2..=3).contains(&required.len()) && !subjects.is_empty() {
        let named = format!("({})", ranking::any_expression(&subjects));
        for row in ranking::documents(db, scope, &named, true, limit * 2)? {
            if articles.insert(row.document) {
                gathered.titled.insert(row.document);
                gathered.rows.push(row);
            }
        }
    }
    if let Some(phrase) = ranking::phrase_expression(required) {
        for row in ranking::documents(db, scope, &phrase, false, window)? {
            gathered.phrase.insert(row.document);
            if articles.insert(row.document) {
                gathered.rows.push(row);
            }
        }
    }
    Ok(gathered)
}

/// Scores full-text candidates by their titles and headings, then reads the
/// leading articles for what their text says.
fn score_text(
    ctx: &Context,
    db: &Connection,
    gathered: Gathered,
    scoring: &ranking::Scoring,
    limit: i64,
) -> Result<Vec<ranking::TextMatch>> {
    let mut lead =
        db.prepare_cached("SELECT d.id,d.pack_id,d.title FROM documents d WHERE d.rowid=?1")?;
    let mut leads = BTreeMap::<i64, Option<Lead>>::new();
    let mut publishers = BTreeMap::<String, HashSet<String>>::new();
    let mut matches = Vec::with_capacity(gathered.rows.len());
    for raw in gathered.rows {
        let rowid = raw.document;
        let lead = match leads.entry(rowid) {
            Slot::Occupied(slot) => slot.get().clone(),
            Slot::Vacant(slot) => {
                let found = lead
                    .query_row([rowid], |row| {
                        Ok(Lead {
                            rowid,
                            id: row.get(0)?,
                            pack_id: row.get(1)?,
                            title: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        })
                    })
                    .optional()?;
                slot.insert(found).clone()
            }
        };
        let Some(lead) = lead else {
            continue;
        };
        let publisher = publishers.entry(lead.pack_id.clone()).or_insert_with(|| {
            let name = ctx
                .catalog
                .docs
                .iter()
                .find(|pack| pack.id == lead.pack_id)
                .map_or("", |pack| pack.name.as_str());
            ranking::words(&format!("{name} {}", lead.pack_id).to_lowercase())
                .map(str::to_owned)
                .collect()
        });
        let in_text = raw.heading.is_none();
        let mut candidate = ranking::TextMatch::new(lead, raw, scoring, publisher);
        if gathered.titled.contains(&rowid) && !candidate.subject {
            // The title shares a word with the query but is about something else.
            continue;
        }
        if in_text {
            candidate.score += i32::from(gathered.phrase.contains(&rowid)) * 14;
        }
        candidate.score += 3 * i32::from(pack_prior(&candidate.lead.pack_id));
        matches.push(candidate);
    }
    matches.sort_by(ranking::TextMatch::compare);
    let mut body = db.prepare_cached("SELECT body FROM documents WHERE rowid=?1")?;
    let mut read = HashSet::new();
    for candidate in &mut matches {
        if read.len() >= limit as usize + 12 {
            break;
        }
        if candidate.heading.is_some() || !read.insert(candidate.lead.rowid) {
            continue;
        }
        let text: Option<String> = body
            .query_row([candidate.lead.rowid], |row| row.get(0))
            .optional()?
            .flatten();
        if let Some(text) = text {
            candidate.read_body(&text, scoring);
        }
    }
    if scoring.fragment {
        // The index was not asked for the letters still being typed, so only
        // articles seen to contain a word that begins with them are matches.
        matches.retain(|candidate| candidate.fragment_seen);
    }
    Ok(matches)
}

fn text_value(
    ctx: &Context,
    db: &Connection,
    candidate: ranking::TextMatch,
) -> Result<Option<Value>> {
    let mut stmt = db.prepare_cached(&format!(
        "SELECT {DOC_COLUMNS},{DOC_PREVIEW} FROM documents d WHERE d.rowid=?1"
    ))?;
    let Some(mut hit) = stmt.query_row([candidate.lead.rowid], summary).optional()? else {
        return Ok(None);
    };
    let focus = if let Some(heading) = candidate.heading {
        if heading.eq_ignore_ascii_case(&hit.title) {
            hit.snippet = compact_excerpt(&hit.snippet);
        } else {
            hit.snippet.clone_from(&heading);
        }
        Some(heading)
    } else if let Some((line, excerpt)) = candidate.line {
        // The excerpt begins at the line that matched, so it shows why.
        hit.snippet = excerpt;
        Some(line)
    } else {
        hit.snippet = compact_excerpt(&hit.snippet);
        None
    };
    let mut value = hit.json(ctx, focus, None);
    if let Some(anchor) = candidate.anchor {
        value["match_anchor"] = json!(anchor);
    }
    Ok(Some(value))
}

/// How well a completion of the typed name answers the query, on the scale
/// used for titles and headings.
fn completion_score(candidate: &AliasMatch) -> i32 {
    let base = if candidate.named {
        84
    } else {
        62 - 4 * candidate.depth.min(3) as i32
    };
    base - candidate.unfinished.min(24) as i32 + 3 * i32::from(pack_prior(&candidate.hit.pack_id))
}

enum Entry {
    Text(ranking::TextMatch),
    Symbol(AliasMatch),
    Alias(Described),
}

impl Entry {
    fn id(&self) -> &str {
        match self {
            Self::Text(candidate) => &candidate.lead.id,
            Self::Symbol(candidate) => &candidate.hit.id,
            Self::Alias(alias) => &alias.hit.id,
        }
    }

    /// The docset a result belongs to. A docset that gathers several
    /// publishers under one roof counts as one docset per publisher.
    fn family(&self) -> String {
        let title = match self {
            Self::Text(candidate) => &candidate.lead.title,
            Self::Symbol(candidate) => &candidate.hit.title,
            Self::Alias(alias) => &alias.hit.title,
        };
        let publisher = title
            .split_once(" · ")
            .map_or("", |(publisher, _)| publisher);
        format!("{}/{publisher}", pack_of(self.id()))
    }
}

/// Orders scored results, letting other docsets in before one docset fills
/// the first page with results that are only slightly better.
fn spread(mut entries: Vec<(i32, f64, Entry)>, most: usize) -> Vec<Entry> {
    entries.sort_by(|(a_score, a_quality, a), (b_score, b_quality, b)| {
        b_score
            .cmp(a_score)
            .then_with(|| a_quality.total_cmp(b_quality))
            .then_with(|| a.id().cmp(b.id()))
    });
    const CONSIDERED: usize = 48;
    let mut remaining: Vec<_> = entries.into_iter().map(Some).collect();
    let mut taken = BTreeMap::<String, i32>::new();
    let mut output = Vec::new();
    let mut first = 0;
    while output.len() < most {
        while first < remaining.len() && remaining[first].is_none() {
            first += 1;
        }
        let best = remaining[first..]
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.as_ref().map(|entry| (index, entry)))
            .take(CONSIDERED)
            .map(|(index, (score, _, entry))| {
                let seen = taken.get(&entry.family()).copied().unwrap_or(0);
                (score - 9 * seen.min(4), std::cmp::Reverse(index))
            })
            .max();
        let Some((_, std::cmp::Reverse(index))) = best else {
            break;
        };
        let (_, _, entry) = remaining[first + index].take().unwrap();
        *taken.entry(entry.family()).or_default() += 1;
        output.push(entry);
    }
    output
}

pub(super) fn search(ctx: &Context, request: &Value) -> Result<Value> {
    let input = request["query"].as_str().unwrap_or("");
    if input.trim().len() > 512 {
        bail!("Search query must be at most 512 bytes");
    }
    let mut db = ctx.database()?;
    let scope = DocScope::new(ctx, &db, request, input, true)?;
    let query = scope.term;
    let limit = request["limit"].as_u64().unwrap_or(30).clamp(1, 100) as i64;
    let mut results = Vec::new();
    if scope.releases == "[]" {
        return Ok(json!(results));
    }
    if query.is_empty() {
        return browse(ctx, &db, request, limit);
    }

    let query = doc_index::without_call(query);
    let called = query != scope.term;
    // A decorator or attribute is documented under the name it wraps.
    let query = query
        .strip_prefix('@')
        .filter(|name| doc_index::symbol_key(name).is_some())
        .unwrap_or(query);
    let symbol_query = ranking::without_generics(query).unwrap_or_else(|| query.to_owned());
    let normalized = symbol_query.to_lowercase();
    let requested_symbol = symbol_pieces(&symbol_query).map(|(symbol, _)| symbol);
    let requested = requested_symbol.map(|symbol| (symbol, symbol_query.as_str()));
    let qualified =
        requested_symbol.is_some() && (symbol_query.contains(['.', '#', ':', '/']) || called);
    let use_prefix = normalized.chars().any(char::is_alphanumeric);
    let (mut exact, mut prefixes): (Vec<_>, Vec<_>) =
        alias_candidates(&db, &scope, &normalized, use_prefix, limit)?
            .into_iter()
            .partition(|candidate| candidate.tier <= 2);
    if exact.is_empty() {
        // A single colon can also separate an installed service's name and command.
        if let Some((owner, term)) = normalized.split_once(':') {
            if !owner.contains(char::is_whitespace) && !term.starts_with([':', '/']) {
                let alias = format!("{owner} {}", term.trim_start());
                exact = alias_candidates(&db, &scope, &alias, false, limit)?;
            }
        }
    }
    if exact.is_empty() {
        // Words can spell out a member and its owner, or one name's parts.
        let mut variants = ranking::joined_symbols(&normalized);
        if variants.is_empty() && !qualified {
            variants = ranking::separated_symbols(&normalized);
        }
        exact = rank_symbols(
            &db,
            &scope,
            variant_rows(&db, &scope, &variants)?,
            true,
            limit,
        )?;
        exact.retain(|candidate| {
            candidate.label.contains(['.', '#', ':', '/', '_'])
                || candidate.label.chars().skip(1).any(char::is_uppercase)
        });
    }

    let mut seen = HashSet::new();
    // A symbol, or an article named by the query, is the answer. An article
    // that merely has the word as an alias competes with headings and titles.
    let (leading, mut aliased): (Vec<_>, Vec<_>) =
        describe_aliases(&db, exact, requested_symbol, &symbol_query, &seen)?
            .into_iter()
            .partition(|alias| qualified || !alias.plain);
    let has_exact_symbol = emit_aliases(ctx, leading, &mut seen, &mut results, limit);
    // A confirmed qualified symbol should not expand to prose with the same words.
    if (qualified && has_exact_symbol) || results.len() >= limit as usize {
        return Ok(json!(results));
    }
    if qualified && prefixes.is_empty() && results.is_empty() {
        let corrected = corrected_symbols(&db, &scope, &normalized, limit)?;
        if append_aliases(
            ctx,
            &db,
            corrected,
            requested,
            &mut seen,
            &mut results,
            limit,
        )? {
            return Ok(json!(results));
        }
        let members = member_candidates(&db, &scope, &symbol_query, limit)?;
        append_aliases(ctx, &db, members, requested, &mut seen, &mut results, limit)?;
    }
    // Keep qualified completions above prose. For general queries, headings and
    // complete word matches can outrank incidental identifier prefixes.
    let short_prefix = !qualified && normalized.chars().count() <= 3;
    // A name in snake or camel case is being typed as a name, so names that
    // complete it lead as well.
    let identifier = requested_symbol.is_some()
        && (symbol_query.contains('_')
            || (symbol_query.chars().skip(1).any(char::is_uppercase)
                && symbol_query.chars().any(char::is_lowercase)));
    if qualified || short_prefix || identifier {
        append_aliases(
            ctx,
            &db,
            std::mem::take(&mut prefixes),
            requested,
            &mut seen,
            &mut results,
            limit,
        )?;
    }
    let terms = ranking::terms(&symbol_query);
    if results.len() >= limit as usize
        || terms.is_empty()
        || query.chars().count() < 2
        || (short_prefix && !results.is_empty())
    {
        emit_aliases(ctx, aliased, &mut seen, &mut results, limit);
        return Ok(json!(results));
    }

    // While a query is being typed its last word may be incomplete. A single
    // trailing letter says too little to be required of an article.
    let typing = !input.ends_with(char::is_whitespace);
    let fragment =
        typing && terms.len() > 1 && terms.last().is_some_and(|word| word.chars().count() <= 2);
    let natural = terms.len() >= 2
        && symbol_query.contains(char::is_whitespace)
        && !symbol_query.contains(['_', '.', ':', '#', '(']);
    // A flag, a dotted name, or a key such as `max_connections` is looked up
    // as written before its words are looked up separately.
    let mut literal = qualified
        || symbol_query.starts_with('-')
        || (terms.len() >= 2 && !symbol_query.contains(char::is_whitespace));
    let window = (limit * 6).clamp(48, 240);
    let mut alternatives: Vec<_> = terms.iter().map(|word| vec![word.clone()]).collect();
    let mut lookup = Lookup {
        typing,
        fragment,
        literal,
        have: results.len() + prefixes.len(),
        window,
        limit,
    };
    doc_index::ensure_sections(&mut db, ctx, &scope.releases)?;
    let mut gathered = gather(&db, &scope, &alternatives, lookup)?;
    if literal && gathered.is_empty() {
        // The name is not written that way anywhere. Look for its words.
        literal = false;
        lookup.literal = false;
        gathered = gather(&db, &scope, &alternatives, lookup)?;
    }
    let mut partial = false;
    // A misspelling usually occurs somewhere in a large library, so a query
    // that finds next to nothing is also tried in its likely spelling.
    let found = gathered.articles();
    if found <= 2 && results.is_empty() && prefixes.is_empty() && aliased.is_empty() {
        if found == 0 && !qualified && requested_symbol.is_some() {
            // A misspelled name is most likely the name one letter away.
            let corrected = corrected_symbols(&db, &scope, &normalized, limit)?;
            append_aliases(
                ctx,
                &db,
                corrected,
                requested,
                &mut seen,
                &mut results,
                limit,
            )?;
        }
        if let Some(corrected) = ranking::corrected_words(&db, &terms)? {
            alternatives = corrected;
            let respelled = gather(&db, &scope, &alternatives, lookup)?;
            if found == 0 {
                gathered = respelled;
            } else {
                // Keep what the query found as typed beside its likely spelling.
                gathered
                    .rows
                    .retain(|row| !gathered.titled.contains(&row.document));
                gathered.titled = respelled.titled;
                gathered.rows.extend(respelled.rows);
                gathered.phrase.extend(respelled.phrase);
            }
        }
        if gathered.is_empty() && alternatives.len() >= 3 {
            // No article has every word. Offer those with the rarest of them,
            // dropping common words until something matches.
            let rarest = ranking::rarest_first(&db, &alternatives)?;
            let mut kept = rarest.len() - 1;
            while gathered.rows.is_empty() {
                let most = ranking::expression(&rarest[..kept], true, ranking::Completion::None);
                gathered.rows = ranking::documents(&db, &scope, &most, false, window)?;
                if kept == 2 {
                    break;
                }
                kept = (kept / 2).max(2);
            }
            partial = true;
        }
    }
    let mut scoring = ranking::Scoring::new(&alternatives, typing, natural, literal);
    scoring.fragment = fragment;
    if literal {
        scoring.written = Some(normalized.clone());
    }
    let mut matches = score_text(ctx, &db, gathered, &scoring, limit)?;
    if partial {
        for candidate in &mut matches {
            candidate.score -= 30;
        }
    }
    ranking::TextMatch::prefer_named_topics(&db, &terms, &mut matches)?;

    // A completion and a full-text match can be the same article. Show it once,
    // as the symbol, at the better of the two positions.
    let mut completions = BTreeMap::<String, i32>::new();
    for candidate in &prefixes {
        completions.insert(
            related_root(&candidate.hit.id).to_owned(),
            completion_score(candidate),
        );
    }
    if !qualified && seen.is_empty() && normalized.contains('_') {
        // A snake-case spelling of a name documented in camel case is a guess
        // that ranks beside the titles and headings containing the words.
        let respelled = [normalized.replace('_', "")];
        for candidate in rank_symbols(
            &db,
            &scope,
            variant_rows(&db, &scope, &respelled)?,
            true,
            limit,
        )? {
            let root = related_root(&candidate.hit.id).to_owned();
            if let Slot::Vacant(slot) = completions.entry(root) {
                slot.insert(95 + 3 * i32::from(pack_prior(&candidate.hit.pack_id)));
                prefixes.push(candidate);
            }
        }
    }
    let mut entries = Vec::with_capacity(matches.len() + prefixes.len());
    for mut candidate in matches {
        let root = related_root(&candidate.lead.id);
        if let Some(score) = completions.get_mut(root) {
            *score = (*score).max(candidate.score);
            continue;
        }
        if aliased
            .iter()
            .any(|alias| related_root(&alias.hit.id) == root)
        {
            candidate.score += 20;
        }
        entries.push((candidate.score, candidate.quality, Entry::Text(candidate)));
    }
    // An alias whose article matched as text is shown at the matching place.
    aliased.retain(|alias| {
        let root = related_root(&alias.hit.id);
        !entries
            .iter()
            .any(|(_, _, entry)| related_root(entry.id()) == root)
    });
    for alias in aliased {
        let score = 100 + 3 * i32::from(pack_prior(&alias.hit.pack_id));
        entries.push((score, f64::MIN, Entry::Alias(alias)));
    }
    for candidate in prefixes {
        let score = completions[related_root(&candidate.hit.id)];
        entries.push((score, f64::MIN, Entry::Symbol(candidate)));
    }
    entries.retain(|(_, _, entry)| !seen.contains(related_root(entry.id())));
    let mut listed = HashSet::new();
    entries.sort_by(|(a, _, _), (b, _, _)| b.cmp(a));
    entries.retain(|(_, _, entry)| match entry {
        // Sections of one article compete for its single place in the results.
        Entry::Text(candidate) => listed.insert(related_root(&candidate.lead.id).to_owned()),
        Entry::Symbol(_) | Entry::Alias(_) => true,
    });
    for entry in spread(entries, limit as usize * 2) {
        if results.len() >= limit as usize {
            break;
        }
        match entry {
            Entry::Text(candidate) => {
                if seen.insert(related_root(&candidate.lead.id).to_owned()) {
                    if let Some(value) = text_value(ctx, &db, candidate)? {
                        results.push(value);
                    }
                }
            }
            Entry::Symbol(candidate) => {
                append_aliases(
                    ctx,
                    &db,
                    vec![candidate],
                    requested,
                    &mut seen,
                    &mut results,
                    limit,
                )?;
            }
            Entry::Alias(alias) => {
                emit_aliases(ctx, vec![alias], &mut seen, &mut results, limit);
            }
        }
    }
    Ok(json!(results))
}

/// A symbol or alias match, with the place in its article that it points to.
struct Described {
    rank: (u8, u8, u8),
    tier: u8,
    /// An alias such as a file name: neither a symbol nor the article's name.
    plain: bool,
    hit: DocSummary,
    focus: Option<String>,
    anchor: Option<String>,
    symbol: Option<String>,
}

fn describe_aliases(
    db: &Connection,
    candidates: Vec<AliasMatch>,
    requested_symbol: Option<&str>,
    typed: &str,
    seen: &HashSet<String>,
) -> Result<Vec<Described>> {
    let bare = !typed.contains(['.', '#', ':', '/']);
    let mut aliases = Vec::new();
    for candidate in candidates {
        if seen.contains(related_root(&candidate.hit.id)) {
            continue;
        }
        let exact = candidate.tier <= 2;
        let (mut hit, mut focus) = if exact || (requested_symbol.is_some() && aliases.len() < 4) {
            resolve_alias(db, candidate.hit, &candidate.label, exact)?
        } else {
            let mut hit = candidate.hit;
            let preview: Option<String> = db
                .query_row(
                    "SELECT substr(body,1,260) FROM documents WHERE id=?1",
                    [&hit.id],
                    |row| row.get(0),
                )
                .optional()?;
            let Some(preview) = preview else {
                continue;
            };
            hit.snippet = compact_excerpt(&preview);
            (hit, None)
        };
        let indexed_target: Option<(String, String)> = db
            .query_row(
                "SELECT a.anchor,coalesce(s.alias,?4) FROM doc_alias_anchors a
                 LEFT JOIN doc_symbol_keys s ON s.document_id=a.document_id AND s.key=?3
                 WHERE a.document_id=?1 AND a.alias=?2",
                params![
                    hit.id,
                    candidate.label.to_lowercase(),
                    doc_index::symbol_key(&candidate.label),
                    candidate.label
                ],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (mut anchor, symbol) = match indexed_target {
            Some((anchor, label)) => (Some(anchor), Some(label)),
            None => (
                None,
                display_symbol(&candidate.label, &hit, focus.as_deref()),
            ),
        };
        if exact && symbol.is_some() && anchor.is_none() {
            if let Some((heading, target)) =
                doc_index::section_focus(db, &hit.id, &candidate.label)?
            {
                let declaration = focus
                    .as_deref()
                    .and_then(|line| find_focus(line, &candidate.label))
                    .is_some_and(|focus| focus.score >= 115);
                if !declaration {
                    focus = Some(heading.clone());
                    hit.snippet = heading;
                }
                anchor = target;
            }
        }
        let spelling_matches = requested_symbol.is_some()
            && symbol
                .as_deref()
                .and_then(symbol_pieces)
                .map(|(symbol, _)| symbol)
                == requested_symbol;
        // Within a bucket, the name as typed leads, then the same name in
        // another case, then members whose own name is spelled as typed.
        // Members that merely carry a bare name keep their docsets' turns.
        let names = [symbol.as_deref().unwrap_or(""), candidate.label.as_str()];
        let as_typed = names
            .iter()
            .any(|name| exact && *name == typed || !exact && name.starts_with(typed));
        let closeness = if as_typed || (bare && !candidate.named) {
            0
        } else if names.iter().any(|name| name.eq_ignore_ascii_case(typed)) {
            1
        } else if spelling_matches {
            2
        } else {
            3
        };
        // A qualified name typed exactly as one docset writes it is that
        // docset's symbol, ahead of another docset's article of the same name.
        let rank = if bare {
            (candidate.bucket, closeness, 0)
        } else {
            (candidate.tier, closeness, u8::from(!candidate.named))
        };
        aliases.push(Described {
            rank,
            tier: candidate.tier,
            plain: symbol.is_none()
                && !candidate.named
                && !candidate.label.contains(char::is_whitespace),
            hit,
            focus,
            anchor,
            symbol,
        });
    }
    // Docsets already take turns within each bucket, and a stable sort keeps that.
    aliases.sort_by_key(|alias| alias.rank);
    Ok(aliases)
}

fn emit_aliases(
    ctx: &Context,
    aliases: Vec<Described>,
    seen: &mut HashSet<String>,
    results: &mut Vec<Value>,
    limit: i64,
) -> bool {
    let mut exact_symbol = false;
    for alias in aliases {
        if results.len() >= limit as usize {
            break;
        }
        if seen.insert(related_root(&alias.hit.id).to_owned()) {
            exact_symbol |= alias.tier <= 2 && alias.symbol.is_some();
            let mut value = alias.hit.json(ctx, alias.focus, alias.symbol);
            if let Some(anchor) = alias.anchor {
                value["match_anchor"] = json!(anchor);
            }
            results.push(value);
        }
    }
    exact_symbol
}

fn append_aliases(
    ctx: &Context,
    db: &Connection,
    candidates: Vec<AliasMatch>,
    requested_symbol: Option<(&str, &str)>,
    seen: &mut HashSet<String>,
    results: &mut Vec<Value>,
    limit: i64,
) -> Result<bool> {
    let typed = requested_symbol.map_or("", |(_, typed)| typed);
    let aliases = describe_aliases(
        db,
        candidates,
        requested_symbol.map(|(symbol, _)| symbol),
        typed,
        seen,
    )?;
    Ok(emit_aliases(ctx, aliases, seen, results, limit))
}

fn corrected_symbols(
    db: &Connection,
    scope: &DocScope,
    query: &str,
    limit: i64,
) -> Result<Vec<AliasMatch>> {
    let Some(key) = doc_index::symbol_key(query) else {
        return Ok(vec![]);
    };
    let components: Vec<_> = key.split('.').collect();
    if components.len() > 6 {
        return Ok(vec![]);
    }
    // A short name on its own has too many neighbors to guess among.
    let minimum = if components.len() == 1 { 5 } else { 3 };
    let mut variants = Vec::new();
    for (i, part) in components.iter().enumerate() {
        for replacement in ranking::one_edit(part, minimum) {
            let mut corrected = components.clone();
            corrected[i] = &replacement;
            variants.push(corrected.join("."));
        }
    }
    let mut rows = variant_rows(db, scope, &variants)?;
    for row in &mut rows {
        row.tier = 2;
    }
    rank_symbols(db, scope, rows, components.len() == 1, limit)
}

fn subsequence(short: &str, long: &str) -> bool {
    let mut letters = long.chars();
    short.chars().all(|ch| letters.any(|other| other == ch))
}

/// A member whose qualified name is not indexed as typed: documented as a
/// section of its owner's article (`Hash#each`), or written with the usual
/// short name of its owner (`np.array`).
fn member_candidates(
    db: &Connection,
    scope: &DocScope,
    query: &str,
    limit: i64,
) -> Result<Vec<AliasMatch>> {
    let Some(key) = doc_index::symbol_key(query) else {
        return Ok(vec![]);
    };
    let Some((owner, leaf)) = key.rsplit_once('.') else {
        return Ok(vec![]);
    };
    let leaf = leaf.split('/').next().unwrap_or(leaf);
    let mut found = Vec::new();
    let mut body = db.prepare_cached("SELECT lower(body) FROM documents WHERE id=?1")?;
    for mut candidate in alias_candidates(db, scope, owner, false, 8)? {
        if !candidate.named {
            continue;
        }
        let documented = doc_index::section_focus(db, &candidate.hit.id, query)?.is_some() || {
            // Some references list members in running text rather than headings.
            let text: Option<String> = body
                .query_row([&candidate.hit.id], |row| row.get(0))
                .optional()?
                .flatten();
            text.is_some_and(|text| {
                [format!("#{leaf}"), format!(".{leaf}"), format!("\n{leaf}(")]
                    .iter()
                    .any(|written| text.contains(written.as_str()))
            })
        };
        if documented {
            candidate.label = doc_index::without_call(query).to_owned();
            candidate.tier = 2;
            candidate.named = false;
            found.push(candidate);
        }
    }
    let short = owner.rsplit('.').next().unwrap_or(owner);
    if !found.is_empty() || short.chars().count() < 2 {
        return Ok(found);
    }
    let mut rows = symbol_rows(db, scope, leaf, false)?;
    rows.retain(|row| {
        row.depth != u32::MAX
            && row.depth > 0
            && doc_index::symbol_key(&row.label).is_some_and(|label| {
                label.rsplit_once('.').is_some_and(|(owners, _)| {
                    owners.split('.').any(|part| {
                        part != short
                            && part.chars().next() == short.chars().next()
                            && subsequence(short, part)
                    })
                })
            })
    });
    for row in &mut rows {
        row.tier = 2;
    }
    rank_symbols(db, scope, rows, true, limit)
}

pub(super) fn preceding_markdown(db: &Connection, id: &str) -> Result<String> {
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

pub(super) fn show(ctx: &Context, request: &Value) -> Result<Value> {
    let input = text_arg(request, "id")?;
    let db = ctx.database()?;
    let direct_scope = DocScope::new(ctx, &db, request, input, false)?;
    let direct: bool = db.query_row(
        &format!("SELECT EXISTS(SELECT 1 FROM documents d WHERE d.id=?2 AND {ALLOWED_RELEASES})"),
        params![direct_scope.releases, direct_scope.term],
        |row| row.get(0),
    )?;
    let (selected, anchor) = if direct {
        (direct_scope.term.to_owned(), None)
    } else {
        let scope = DocScope::new(ctx, &db, request, input, true)?;
        let matches = alias_candidates(&db, &scope, &scope.term.to_lowercase(), false, 2)?;
        let best_tier = matches.first().map(|candidate| candidate.tier);
        let mut matches = matches
            .into_iter()
            .filter(|candidate| Some(candidate.tier) == best_tier);
        match (matches.next(), matches.next()) {
            (Some(candidate), None) => {
                let (hit, anchor) = resolve_alias(&db, candidate.hit, &candidate.label, true)?;
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
                "Several documents match this reference. Use a fully qualified symbol, --version, or an exact result ID."
            ),
        }
    };
    let sql = format!("SELECT {DOC_COLUMNS},'',d.body,d.html FROM documents d WHERE d.id=?1");
    let (hit, body, mut html): (DocSummary, String, Option<String>) =
        db.query_row(&sql, [selected], |row| {
            Ok((summary(row)?, row.get(8)?, row.get(9)?))
        })?;
    let mut focus = anchor.or_else(|| {
        let query = request["focus_text"]
            .as_str()
            .or_else(|| request["query"].as_str())?;
        // Direct result selections may pass the already matched declaration instead of the typed scope.
        if request["focus_text"].as_str().is_some() && body.contains(query) {
            return Some(query.chars().take(180).collect());
        }
        let scoped = DocScope::new(ctx, &db, request, query, false).ok()?;
        find_focus(&body, scoped.term).map(|focus| focus.line.chars().take(180).collect())
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
    let mut terminal_markdown = None;
    let want_markdown = request["terminal_format"].as_bool() == Some(true);
    if (html.is_none() || want_markdown) && markdown::matches(&hit.pack_id, path, rules) {
        let preceding = preceding_markdown(&db, &hit.id)?;
        let source = markdown::with_continuation(&body, &preceding);
        if html.is_none() {
            let (rendered, visible_focus) =
                markdown::render(&source, &hit.source_url, focus.as_deref());
            html = Some(rendered);
            focus = visible_focus;
        }
        if want_markdown {
            terminal_markdown = Some(source.into_owned());
        }
    }
    let parts = continuation_ids(&db, &hit.id)?;
    let title = hit.display_title(ctx);
    let mut value = json!({
        "id": hit.id, "pack_id": hit.pack_id, "title": title, "stack": hit.stack,
        "version": hit.version, "source_url": hit.source_url, "body": body, "html": html,
        "category": hit.category,
    });
    if let Some(focus) = focus {
        value["focus_text"] = json!(focus);
    }
    if let Some(markdown) = terminal_markdown {
        value["terminal_markdown"] = json!(markdown);
    }
    if let Some(index) = parts
        .iter()
        .position(|id| *id == hit.id)
        .filter(|_| parts.len() > 1)
    {
        value["part"] = json!(index + 1);
        value["parts"] = json!(parts.len());
        if let Some(previous) = index.checked_sub(1).and_then(|index| parts.get(index)) {
            value["previous_id"] = json!(previous);
        }
        if let Some(next) = parts.get(index + 1) {
            value["next_id"] = json!(next);
        }
    }
    Ok(value)
}

pub(super) fn continuation_ids(db: &Connection, id: &str) -> Result<Vec<String>> {
    let root = related_root(id);
    let mut stmt = db.prepare_cached(
        "SELECT id FROM documents WHERE id=?1 OR (id>=?2 AND id<?3)
         ORDER BY CAST(substr(id,length(?1)+8) AS INTEGER),id",
    )?;
    let rows = stmt.query_map(
        params![root, format!("{root}::part-"), format!("{root}::part.")],
        |row| row.get::<_, String>(0),
    )?;
    let mut parts = Vec::new();
    for row in rows {
        let id = row?;
        if related_root(&id) == root {
            parts.push(id);
        }
    }
    Ok(parts)
}

pub(super) fn focus_for_link(
    db: &Connection,
    id: &str,
    fragment: &str,
) -> Result<(String, Option<String>)> {
    let hit = db.query_row(
        &format!("SELECT {DOC_COLUMNS},'' FROM documents d WHERE d.id=?1"),
        [id],
        summary,
    )?;
    let (hit, focus) = resolve_alias(db, hit, fragment, true)?;
    Ok((hit.id, focus))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_sort_by_number_then_letter_ignoring_markup() {
        let mut titles = [
            "10. Brief tour",
            "2. Using the interpreter",
            "<StrictMode>",
            "1. Whetting your appetite",
            "What's New In Python 3.10",
            "What's New In Python 3.9",
            "`APIRouter` class",
            "Zlib",
            "007 notes",
        ];
        titles.sort_by_key(|title| natural_key(title));
        assert_eq!(
            titles,
            [
                "1. Whetting your appetite",
                "2. Using the interpreter",
                "007 notes",
                "10. Brief tour",
                "`APIRouter` class",
                "<StrictMode>",
                "What's New In Python 3.9",
                "What's New In Python 3.10",
                "Zlib",
            ]
        );
    }

    #[test]
    fn docsets_take_turns_within_a_rank() {
        let ranked = vec![
            (0, "java"),
            (1, "java"),
            (1, "java"),
            (1, "java"),
            (1, "ruby"),
            (1, "elixir"),
            (2, "java"),
        ];
        let turns = take_turns(ranked, |(rank, _)| *rank, |(_, pack)| pack);
        let packs: Vec<_> = turns.iter().map(|(_, pack)| *pack).collect();
        assert_eq!(
            packs,
            ["java", "java", "ruby", "elixir", "java", "java", "java"]
        );
    }

    #[test]
    fn articles_are_recognized_by_the_name_they_document() {
        for (title, label) in [
            ("HashMap in std::collections", "std::collections::HashMap"),
            ("json — JSON encoder and decoder", "json"),
            ("`FastAPI` class", "fastapi.FastAPI"),
            ("EventTarget: addEventListener() method", "addEventListener"),
            ("Array#map", "Array#map"),
            ("View.padding(_:)", "view.padding"),
        ] {
            assert!(names_article(title, label), "{title}");
        }
        for (title, label) in [
            ("Enum", "Enum.map/2"),
            ("json — JSON encoder and decoder", "json.dumps"),
            ("Response caching middleware in ASP.NET Core", "middleware"),
        ] {
            assert!(!names_article(title, label), "{title}");
        }
    }
}
