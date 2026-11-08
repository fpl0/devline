use super::{summary, DocScope, DocSummary, ALLOWED_RELEASES, DOC_COLUMNS, DOC_PREVIEW};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::{BTreeSet, HashSet};

const MAX_TERMS: usize = 12;

pub(super) fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|word| !word.is_empty())
}

pub(super) fn terms(query: &str) -> Vec<String> {
    let words: Vec<_> = words(query)
        .take(MAX_TERMS * 2)
        .map(str::to_lowercase)
        .collect();
    if words.len() < 3 {
        return words;
    }
    let significant: Vec<_> = words
        .iter()
        .filter(|word| {
            !matches!(
                word.as_str(),
                "a" | "an"
                    | "are"
                    | "can"
                    | "do"
                    | "does"
                    | "how"
                    | "i"
                    | "is"
                    | "the"
                    | "to"
                    | "what"
                    | "when"
                    | "where"
                    | "which"
                    | "why"
                    | "you"
            )
        })
        .take(MAX_TERMS)
        .cloned()
        .collect();
    if significant.is_empty() {
        words.into_iter().take(MAX_TERMS).collect()
    } else {
        significant
    }
}

/// Spellings for a pasted type such as Vec<T> or List<String>.Add(value).
pub(super) fn without_generics(query: &str) -> Option<String> {
    let (name, arguments) = query.split_once('<')?;
    let name = name.trim_end().trim_end_matches("::");
    super::doc_index::symbol_key(name)?;
    let mut depth = 1;
    for (index, ch) in arguments.char_indices() {
        if ch == '<' {
            depth += 1;
        } else if ch == '>' {
            depth -= 1;
            if depth == 0 {
                let suffix = &arguments[index + 1..];
                if suffix.is_empty() || suffix.starts_with(['.', '#']) || suffix.starts_with("::") {
                    return Some(format!("{name}{suffix}"));
                }
                return None;
            }
        }
    }
    None
}

/// Space-delimited names typed in the search box often denote an owner and member,
/// or a single identifier with its case boundaries spelled out.
pub(super) fn joined_symbols(query: &str) -> Vec<String> {
    let words: Vec<_> = query.split_whitespace().take(6).collect();
    if !(2..=5).contains(&words.len())
        || !words
            .iter()
            .all(|word| super::doc_index::symbol_component(word))
    {
        return vec![];
    }
    let [owner, members @ ..] = words.as_slice() else {
        return vec![];
    };
    let mut variants = vec![format!("{owner}.{}", members.concat()), words.concat()];
    if members.len() > 1 {
        variants.push(format!("{owner}.{}", members.join("_")));
    }
    variants
}

pub(super) fn one_edit(word: &str, minimum: usize) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    if !(minimum..=24).contains(&word.len()) || !word.bytes().all(|ch| ch.is_ascii_lowercase()) {
        return candidates;
    }
    for i in 0..=word.len() {
        let (before, after) = word.split_at(i);
        for letter in 'a'..='z' {
            candidates.insert(format!("{before}{letter}{after}"));
            if let Some(tail) = after.get(1..) {
                candidates.insert(format!("{before}{letter}{tail}"));
            }
        }
        if let Some(tail) = after.get(1..) {
            candidates.insert(format!("{before}{tail}"));
        }
        if let Some(tail) = after.get(2..) {
            candidates.insert(format!("{before}{}{}{tail}", &after[1..2], &after[..1]));
        }
    }
    candidates.remove(word);
    candidates
}

pub(super) fn corrected_words(
    db: &Connection,
    terms: &[String],
) -> Result<Option<Vec<Vec<String>>>> {
    if terms.len() > 6 {
        return Ok(None);
    }
    db.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS temp.doc_search_vocab USING fts5vocab(main, docs_fts, row)")?;
    let mut frequency = db.prepare_cached("SELECT doc FROM temp.doc_search_vocab WHERE term=?1")?;
    let mut changed = false;
    let mut corrected = Vec::new();
    for word in terms {
        if frequency.exists([word])? {
            corrected.push(vec![word.clone()]);
            continue;
        }
        let mut neighbors = Vec::new();
        for neighbor in one_edit(word, 4) {
            let mut rows = frequency.query([&neighbor])?;
            if let Some(row) = rows.next()? {
                neighbors.push((row.get::<_, u64>(0)?, neighbor));
            }
        }
        neighbors.sort_unstable();
        neighbors.reverse();
        if neighbors.is_empty() {
            return Ok(None);
        }
        changed = true;
        corrected.push(
            neighbors
                .into_iter()
                .take(5)
                .map(|(_, word)| word)
                .collect(),
        );
    }
    Ok(changed.then_some(corrected))
}

fn forms(term: &str) -> Vec<String> {
    let mut forms = vec![term.to_owned()];
    if term.len() < 3 || !term.bytes().all(|ch| ch.is_ascii_lowercase()) {
        return forms;
    }
    if let Some(base) = term.strip_suffix("ies").filter(|base| base.len() >= 3) {
        forms.push(format!("{base}y"));
    } else if let Some(base) = term.strip_suffix('s').filter(|base| base.len() >= 3) {
        forms.push(base.to_owned());
    } else if let Some(base) = term
        .strip_suffix('y')
        .filter(|base| !base.ends_with(['a', 'e', 'i', 'o', 'u']))
    {
        forms.push(format!("{base}ies"));
    } else {
        forms.push(format!("{term}s"));
    }
    if let Some(base) = term.strip_suffix("ing").filter(|base| base.len() >= 4) {
        forms.push(base.to_owned());
        forms.push(format!("{base}e"));
    } else if term.len() >= 4 {
        let base = term.strip_suffix('e').unwrap_or(term);
        forms.push(format!("{base}ing"));
        forms.push(format!("{base}ed"));
    }
    forms
}

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

pub(super) fn expression(terms: &[Vec<String>], flexible: bool, prefix: bool) -> String {
    terms
        .iter()
        .enumerate()
        .map(|(i, choices)| {
            let mut options: Vec<_> = choices
                .iter()
                .flat_map(|word| {
                    if flexible {
                        forms(word)
                    } else {
                        vec![word.clone()]
                    }
                })
                .map(|word| quoted(&word))
                .collect();
            if prefix
                && i + 1 == terms.len()
                && choices[0].chars().count() >= if terms.len() > 1 { 1 } else { 3 }
            {
                options.push(format!("{}*", quoted(&choices[0])));
            }
            format!("({})", options.join(" OR "))
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub(super) struct TextMatch {
    pub hit: DocSummary,
    pub heading: Option<String>,
    pub anchor: Option<String>,
    score: i32,
    quality: f64,
}

fn contains_phrase(words: &[&str], terms: &[Vec<String>]) -> bool {
    words.windows(terms.len()).any(|slice| {
        slice
            .iter()
            .zip(terms)
            .all(|(word, choices)| choices.iter().any(|choice| word == choice))
    })
}

fn evidence(words: &[&str], terms: &[Vec<String>]) -> (i32, bool, bool) {
    let coverage = terms
        .iter()
        .filter(|choices| {
            choices
                .iter()
                .any(|choice| words.contains(&choice.as_str()))
        })
        .count() as i32;
    let phrase = contains_phrase(words, terms);
    (coverage, phrase, phrase && words.len() == terms.len())
}

impl TextMatch {
    fn new(
        hit: DocSummary,
        heading: Option<String>,
        anchor: Option<String>,
        quality: f64,
        terms: &[Vec<String>],
    ) -> Self {
        let title = hit.title.to_lowercase();
        let title_tokens: Vec<_> = words(&title).collect();
        let (title_words, title_phrase, exact_title) = evidence(&title_tokens, terms);
        let mut score =
            45 * title_words + i32::from(title_phrase) * 26 + i32::from(exact_title) * 35;
        if let Some(heading) = &heading {
            // An FTS heading hit has matching terms, possibly with stemming or
            // identifier splitting. Count an article's repeated title only once.
            let heading = heading.to_lowercase();
            if heading != title {
                let tokens: Vec<_> = heading
                    .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
                    .filter(|word| !word.is_empty())
                    .collect();
                let (heading_words, heading_phrase, exact_heading) = evidence(&tokens, terms);
                score += 22
                    + 20 * terms.len() as i32
                    + 4 * heading_words
                    + i32::from(heading_phrase) * 18
                    + i32::from(exact_heading) * 25;
            }
        } else {
            let snippet = hit.snippet.to_lowercase();
            let tokens: Vec<_> = words(&snippet).collect();
            let (snippet_words, snippet_phrase, _) = evidence(&tokens, terms);
            let nearby_pairs = terms
                .windows(2)
                .filter(|pair| contains_phrase(&tokens, pair))
                .count() as i32;
            score += snippet_words * 3 + nearby_pairs * 12 + i32::from(snippet_phrase) * 15;
        }
        Self {
            hit,
            heading,
            anchor,
            score,
            quality,
        }
    }

    pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
        b.score
            .cmp(&a.score)
            .then_with(|| a.quality.total_cmp(&b.quality))
            .then_with(|| a.hit.id.cmp(&b.hit.id))
    }

    pub fn prefer_named_topics(db: &Connection, terms: &[String], hits: &mut [Self]) -> Result<()> {
        if hits.is_empty() || terms.len() < 3 {
            return Ok(());
        }
        // A named command or topic followed by a detail should win over pages
        // that only mention those words, provided the full query matched too.
        let aliases: Vec<_> = (2..terms.len())
            .map(|count| terms[..count].join(" "))
            .collect();
        let mut stmt = db.prepare_cached(
            "SELECT a.document_id FROM json_each(?1) prefix
             JOIN aliases a ON a.alias=prefix.value",
        )?;
        let named = stmt
            .query_map([serde_json::to_string(&aliases)?], |row| {
                row.get::<_, String>(0)
            })?
            .collect::<rusqlite::Result<HashSet<_>>>()?;
        for hit in hits {
            if named.contains(&hit.hit.id) {
                hit.score += 35;
            }
        }
        Ok(())
    }
}

pub(super) fn documents(
    db: &Connection,
    scope: &DocScope,
    terms: &[Vec<String>],
    prefix: bool,
    window: i64,
) -> Result<Vec<TextMatch>> {
    let mut expression = format!("{{title body}} : ({})", expression(terms, true, prefix));
    if !scope.stack.is_empty() {
        expression.push_str(&format!(" AND stack : {}", quoted(scope.stack)));
    }
    let category = scope.category_clause;
    // Rank candidate articles before constructing excerpts. This avoids running snippet()
    // against every mention of a common search term.
    let sql = format!(
        "WITH matches AS MATERIALIZED (
           SELECT d.rowid AS id, bm25(docs_fts,12.0,1.0,0.0) AS quality
           FROM docs_fts JOIN documents d ON d.rowid=docs_fts.rowid
           WHERE docs_fts MATCH ?2 AND {ALLOWED_RELEASES} {category}
           ORDER BY quality, d.id LIMIT ?3
         )
         SELECT {DOC_COLUMNS},
             coalesce(nullif(snippet(docs_fts,1,'','',' … ',28),''),{DOC_PREVIEW}),
             matches.quality
         FROM matches JOIN documents d ON d.rowid=matches.id
         JOIN docs_fts ON docs_fts.rowid=matches.id
         WHERE docs_fts MATCH ?2"
    );
    let mut stmt = db.prepare_cached(&sql)?;
    let hits = stmt
        .query_map(params![scope.releases, expression, window], |row| {
            Ok(TextMatch::new(
                summary(row)?,
                None,
                None,
                row.get(8)?,
                terms,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(hits)
}

pub(super) fn sections(
    db: &Connection,
    scope: &DocScope,
    terms: &[Vec<String>],
    prefix: bool,
    window: i64,
) -> Result<Vec<TextMatch>> {
    let expression = expression(terms, false, prefix);
    let category = scope.category_clause;
    let sql = format!(
        "SELECT {DOC_COLUMNS},{DOC_PREVIEW},s.heading,s.anchor,bm25(doc_sections_fts,2.0,1.0)
         FROM doc_sections_fts JOIN doc_sections s ON s.rowid=doc_sections_fts.rowid
         JOIN documents d ON d.rowid=s.document_rowid
         WHERE doc_sections_fts MATCH ?2 AND {ALLOWED_RELEASES} {category}
         ORDER BY 11, d.id, s.rowid LIMIT ?3"
    );
    let mut stmt = db.prepare_cached(&sql)?;
    let hits = stmt
        .query_map(params![scope.releases, expression, window], |row| {
            Ok(TextMatch::new(
                summary(row)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
                terms,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(hits)
}
