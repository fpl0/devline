use crate::{doc_index, markdown, text_arg, updates::compare_doc_releases, Context};
use anyhow::{bail, Context as _, Result};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

struct DocScope<'a> {
    stack: &'a str,
    term: &'a str,
    releases: String,
    category_clause: &'static str,
}

impl<'a> DocScope<'a> {
    fn new(
        ctx: &Context,
        db: &Connection,
        request: &'a Value,
        input: &'a str,
        latest: bool,
    ) -> Result<Self> {
        let mut stmt = db.prepare_cached(
            "SELECT p.id,p.version,(SELECT stack FROM documents d WHERE d.pack_id=p.id AND d.version=p.version LIMIT 1)
             FROM packs p",
        )?;
        let installed = stmt
            .query_map([], |row| {
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
        let stack = match request["stack"].as_str() {
            Some("all" | "") => "",
            Some(stack) => stack,
            None if selectors.is_empty() => request["project_stack"].as_str().unwrap_or(""),
            None => "",
        };
        let pack = request["pack_id"].as_str().unwrap_or("");
        let version = request["version"].as_str().unwrap_or("");
        let eligible = |id: &str, release: &str, family: &str| {
            (stack.is_empty() || stack == family)
                && (pack.is_empty() || pack == id)
                && (version.is_empty() || version == release)
                && (selectors.is_empty()
                    || selectors
                        .iter()
                        .any(|selector| selector.matches(id, release, family)))
        };
        let mut head = BTreeMap::<&str, &str>::new();
        for (id, release, family) in &installed {
            if eligible(id, release, family) {
                let current = head.entry(id).or_insert(release);
                if compare_doc_releases(release, current).is_gt() {
                    *current = release;
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
        let category_clause = match request["category"].as_str().unwrap_or_default() {
            "" | "all" => "",
            "guide" => "AND d.category='guide'",
            "api" => "AND d.category='api'",
            "compatibility" => "AND d.category='compatibility'",
            "reference" => "AND d.category='reference'",
            _ => bail!("Choose guide, api, compatibility, reference, or all for category"),
        };
        Ok(Self {
            stack,
            term,
            releases: serde_json::to_string(&releases)?,
            category_clause,
        })
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

fn alias_candidates(
    db: &Connection,
    scope: &DocScope,
    query: &str,
    prefix: bool,
    limit: i64,
) -> Result<Vec<AliasMatch>> {
    // Bounds are bound parameters, so punctuation in names is literal. The existing
    // aliases_lookup index serves both equality and prefix matches.
    let upper = prefix_end(query);
    let matches = if prefix && !upper.is_empty() {
        "a.alias>=?2 AND a.alias<?3"
    } else {
        "a.alias=?2"
    };
    let category = scope.category_clause;
    let sql = format!(
        "SELECT {DOC_COLUMNS},'',a.alias FROM aliases a INDEXED BY aliases_lookup
         JOIN documents d ON d.id=a.document_id
         WHERE {matches} AND {ALLOWED_RELEASES} {category}
         ORDER BY a.alias LIMIT ?4"
    );
    let mut stmt = db.prepare(&sql)?;
    let window = (limit * 6).max(48);
    let mut results = stmt
        .query_map(params![scope.releases, query, upper, window], |row| {
            let label = row.get::<_, String>(8)?;
            let tier = if label == query { 0 } else { 4 };
            Ok(AliasMatch {
                hit: summary(row)?,
                label,
                tier,
                depth: u32::MAX,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    if let Some(key) = doc_index::symbol_key(query) {
        let upper = prefix_end(&key);
        let range = if prefix && key.chars().count() >= 3 && !upper.is_empty() {
            "s.key>=?2 AND s.key<?3"
        } else {
            "s.key=?2"
        };
        let sql = format!(
            "SELECT {DOC_COLUMNS},'',s.alias,s.depth,s.key=?2
             FROM doc_symbol_keys s JOIN documents d ON d.id=s.document_id
             WHERE {range} AND {ALLOWED_RELEASES} {category}
             ORDER BY s.key LIMIT ?4"
        );
        let mut stmt = db.prepare(&sql)?;
        let rows = stmt.query_map(params![scope.releases, key, upper, window], |row| {
            let depth = row.get::<_, u32>(9)?;
            let label = row.get::<_, String>(8)?;
            let tier = if !row.get::<_, bool>(10)? {
                5
            } else if label.eq_ignore_ascii_case(query) {
                0
            } else if depth == 0 {
                1
            } else {
                2
            };
            Ok(AliasMatch {
                hit: summary(row)?,
                label,
                tier,
                depth,
            })
        })?;
        for row in rows {
            results.push(row?);
        }
    }
    results.sort_by(|a, b| {
        (
            a.tier,
            api_priority(&a.hit.id),
            a.depth,
            a.label.len(),
            &a.hit.title,
            &a.hit.id,
        )
            .cmp(&(
                b.tier,
                api_priority(&b.hit.id),
                b.depth,
                b.label.len(),
                &b.hit.title,
                &b.hit.id,
            ))
    });
    let mut seen = HashSet::new();
    results.retain(|hit| seen.insert(related_root(&hit.hit.id).to_owned()));
    results.truncate(limit as usize);
    Ok(results)
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
        } else if hit.title.eq_ignore_ascii_case(segment) {
            Some(hit.title.as_str())
        } else {
            lower.match_indices(segment).find_map(|(start, _)| {
                let end = start + segment.len();
                let word = |ch: char| ch.is_alphanumeric() || ch == '_';
                let suffix = &lower[end..];
                (!lower[..start].ends_with(word)
                    && !suffix.starts_with(word)
                    && (leaf || suffix.starts_with(['.', '#', ':'])))
                .then(|| anchor.get(start..end))
                .flatten()
            })
        };
        spelling.push_str(cased.unwrap_or(segment));
        spelling.push_str(&component[segment.len()..]);
    }
    Some(spelling)
}

pub(super) fn search(ctx: &Context, request: &Value) -> Result<Value> {
    let input = request["query"].as_str().unwrap_or("");
    if input.trim().len() > 512 {
        bail!("Search query must be at most 512 bytes");
    }
    let db = ctx.database()?;
    let scope = DocScope::new(ctx, &db, request, input, true)?;
    let query = scope.term;
    let category = scope.category_clause;
    let limit = request["limit"].as_u64().unwrap_or(30).clamp(1, 100) as i64;
    let mut results = Vec::new();
    if query.is_empty() {
        let offset = request["offset"].as_u64().unwrap_or(0).min(100_000) as i64;
        let sql = format!(
            "SELECT {DOC_COLUMNS},{DOC_PREVIEW} FROM json_each(?1) r JOIN documents d
             ON d.pack_id=substr(r.value,1,instr(r.value,'@')-1)
             AND d.version=substr(r.value,instr(r.value,'@')+1)
             WHERE d.id NOT GLOB '*::part-[0-9]*' {category}
             ORDER BY lower(d.title), d.id LIMIT ?2 OFFSET ?3"
        );
        let mut stmt = db.prepare(&sql)?;
        for row in stmt.query_map(params![scope.releases, limit, offset], summary)? {
            results.push(row?.json(ctx, None, None));
        }
        return Ok(json!(results));
    }

    let normalized = query.to_lowercase();
    let mut seen = HashSet::new();
    let use_prefix = normalized.chars().any(char::is_alphanumeric);
    let requested_symbol = symbol_pieces(query).map(|(symbol, _)| symbol);
    let qualified = requested_symbol.is_some()
        && (query.contains(['.', '#', ':', '/']) || doc_index::without_call(query) != query);
    let mut exact_symbol = false;
    let mut aliases = Vec::new();
    for candidate in alias_candidates(&db, &scope, &normalized, use_prefix, limit)? {
        let exact = candidate.tier <= 2;
        let (hit, anchor) = if exact || qualified && aliases.len() < 4 {
            resolve_alias(&db, candidate.hit, &candidate.label, exact)?
        } else {
            (candidate.hit, None)
        };
        if seen.insert(related_root(&hit.id).to_owned()) {
            let symbol = display_symbol(&candidate.label, &hit, anchor.as_deref());
            exact_symbol |= exact && symbol.is_some();
            let spelling_matches = requested_symbol.is_some()
                && symbol
                    .as_deref()
                    .and_then(symbol_pieces)
                    .map(|(symbol, _)| symbol)
                    == requested_symbol;
            aliases.push((
                candidate.tier,
                !spelling_matches,
                hit.json(ctx, anchor, symbol),
            ));
        }
    }
    aliases.sort_by_key(|(tier, different_spelling, _)| (*tier, *different_spelling));
    results.extend(aliases.into_iter().map(|(_, _, hit)| hit));
    // A confirmed qualified symbol should not expand to prose with the same words.
    if results.len() < limit as usize && !(qualified && exact_symbol) && query.chars().count() > 1 {
        let terms: Vec<_> = doc_index::without_call(query)
            .split_whitespace()
            .filter(|term| term.chars().any(char::is_alphanumeric))
            .take(12)
            .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
            .collect();
        if !terms.is_empty() {
            let terms = terms.join(" AND ");
            for (columns, snippet) in [
                ("title", DOC_PREVIEW),
                ("{title body}", "snippet(docs_fts,1,'','',' … ',28)"),
            ] {
                let mut expression = format!("{columns} : ({terms})");
                if !scope.stack.is_empty() {
                    expression.push_str(&format!(
                        " AND stack : \"{}\"",
                        scope.stack.replace('"', "\"\"")
                    ));
                }
                let sql = format!(
                    "SELECT {DOC_COLUMNS},{snippet} FROM docs_fts
                     JOIN documents d ON d.rowid=docs_fts.rowid
                     WHERE docs_fts MATCH ?2 AND {ALLOWED_RELEASES} {category}
                     ORDER BY bm25(docs_fts,8.0,1.0,0.0), d.id LIMIT ?3"
                );
                let mut stmt = db.prepare(&sql)?;
                for row in
                    stmt.query_map(params![scope.releases, expression, limit * 3], summary)?
                {
                    let hit = row?;
                    if seen.insert(related_root(&hit.id).to_owned()) {
                        results.push(hit.json(ctx, None, None));
                        if results.len() == limit as usize {
                            return Ok(json!(results));
                        }
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
                "Several documents match this reference. Use --version or an exact result id."
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
