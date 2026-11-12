use super::{DocScope, Lead};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::{BTreeSet, HashSet};

const MAX_TERMS: usize = 12;

pub(super) fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|word| !word.is_empty())
}

fn filler(word: &str) -> bool {
    matches!(
        word,
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
}

pub(super) fn terms(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let words: Vec<_> = words(query)
        .take(MAX_TERMS * 2)
        .map(str::to_lowercase)
        .filter(|word| seen.insert(word.clone()))
        .collect();
    if words.len() < 3 {
        return words;
    }
    let mut significant: Vec<_> = words
        .iter()
        .filter(|word| !filler(word))
        .take(MAX_TERMS)
        .cloned()
        .collect();
    // In a longer query, single letters select nothing and cost the most.
    if significant.len() > 3 {
        let letters = |word: &String| word.chars().count() == 1;
        if significant.iter().all(letters) {
            significant.truncate(3);
        } else {
            significant.retain(|word| !letters(word));
        }
    }
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
    variants.push(words.join("_"));
    variants
}

/// Snake-case spellings of a name typed without its separators, such as
/// `readtext` for `read_text`. Only spellings that exist are ever returned
/// by the index, so the list can be generous.
pub(super) fn separated_symbols(query: &str) -> Vec<String> {
    let length = query.len();
    if !(4..=24).contains(&length)
        || !query
            .bytes()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    {
        return vec![];
    }
    let mut variants = Vec::new();
    for first in 1..length {
        variants.push(format!("{}_{}", &query[..first], &query[first..]));
        for second in first + 1..length {
            variants.push(format!(
                "{}_{}_{}",
                &query[..first],
                &query[first..second],
                &query[second..]
            ));
        }
    }
    variants
}

pub(super) fn one_edit(word: &str, minimum: usize) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    if !(minimum..=24).contains(&word.len())
        || !word
            .bytes()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == b'_')
    {
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

/// Replaces words the library hardly knows with close spellings it knows well.
/// A misspelling usually exists somewhere in a large library, so a word counts
/// as unknown when a neighbor is far more common.
pub(super) fn corrected_words(
    db: &Connection,
    terms: &[String],
) -> Result<Option<Vec<Vec<String>>>> {
    if terms.len() > 6 {
        return Ok(None);
    }
    db.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS temp.doc_search_vocab USING fts5vocab(main, docs_fts, row)")?;
    let mut frequency = db.prepare_cached("SELECT doc FROM temp.doc_search_vocab WHERE term=?1")?;
    let mut count = |word: &str| -> Result<u64> {
        let mut rows = frequency.query([word])?;
        Ok(match rows.next()? {
            Some(row) => row.get(0)?,
            None => 0,
        })
    };
    let mut changed = false;
    let mut corrected = Vec::new();
    for word in terms {
        let known = count(word)?;
        if known >= 40 {
            corrected.push(vec![word.clone()]);
            continue;
        }
        let mut neighbors = Vec::new();
        for neighbor in one_edit(word, 4) {
            let found = count(&neighbor)?;
            if found > 0 && found >= known.saturating_mul(20) {
                neighbors.push((found, neighbor));
            }
        }
        if neighbors.is_empty() && word.len() >= 6 && word.is_ascii() {
            // Two slips in a longer word: compare the words that begin alike.
            let prefix = &word[..if word.len() >= 8 { 4 } else { 3 }];
            let mut vocabulary = db.prepare_cached(
                "SELECT term,doc FROM temp.doc_search_vocab WHERE term>=?1 AND term<?2",
            )?;
            let mut rows = vocabulary.query([prefix, &format!("{prefix}\u{10ffff}")])?;
            while let Some(row) = rows.next()? {
                let term = row.get_ref(0)?.as_str()?;
                if term.len().abs_diff(word.len()) <= 2 && within_two_edits(word, term) {
                    let found: u64 = row.get(1)?;
                    if found >= known.saturating_mul(20).max(40) {
                        neighbors.push((found, term.to_owned()));
                    }
                }
            }
        }
        neighbors.sort_unstable();
        neighbors.reverse();
        if neighbors.is_empty() {
            if known == 0 {
                return Ok(None);
            }
            corrected.push(vec![word.clone()]);
            continue;
        }
        changed = true;
        // The best spelling leads so that scoring and excerpts use it.
        let mut choices: Vec<_> = neighbors
            .into_iter()
            .take(5)
            .map(|(_, word)| word)
            .collect();
        if known > 0 {
            choices.push(word.clone());
        }
        corrected.push(choices);
    }
    Ok(changed.then_some(corrected))
}

/// Whether two words differ by at most two insertions, deletions,
/// substitutions, or swaps of adjacent letters.
fn within_two_edits(a: &str, b: &str) -> bool {
    let (a, b): (Vec<_>, Vec<_>) = (a.chars().collect(), b.chars().collect());
    let mut before: Vec<usize> = Vec::new();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut current = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let substitution = previous[j - 1] + usize::from(a[i - 1] != b[j - 1]);
            current[j] = substitution.min(previous[j] + 1).min(current[j - 1] + 1);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                current[j] = current[j].min(before[j - 2] + 1);
            }
        }
        before = std::mem::replace(&mut previous, current);
    }
    previous[b.len()] <= 2
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
    // Nouns and verbs of one topic: migrate and migration, deploy and deployment.
    if term.len() >= 5 {
        if let Some(base) = term
            .strip_suffix("ions")
            .or_else(|| term.strip_suffix("ion"))
            .filter(|base| base.len() >= 4)
        {
            forms.push(base.to_owned());
            forms.push(format!("{base}e"));
            forms.push(format!("{base}ing"));
        } else if let Some(base) = term
            .strip_suffix("ments")
            .or_else(|| term.strip_suffix("ment"))
            .filter(|base| base.len() >= 5)
        {
            forms.push(base.to_owned());
            forms.push(format!("{base}ing"));
        } else if !term.ends_with("ing") && !term.ends_with('s') {
            let base = term.strip_suffix('e').unwrap_or(term);
            forms.push(format!("{base}ion"));
            forms.push(format!("{base}ions"));
            forms.push(format!("{term}ment"));
        }
    }
    forms
}

/// A light stem used to compare query words with titles and headings.
pub(super) fn stem(word: &str) -> String {
    let mut stem = word.to_owned();
    if stem.len() < 4 || !stem.bytes().all(|ch| ch.is_ascii_lowercase()) {
        return stem;
    }
    if stem.len() >= 5 && stem.ends_with("ies") {
        stem.truncate(stem.len() - 3);
        stem.push('y');
    } else if stem.ends_with("sses") {
        stem.truncate(stem.len() - 2);
    } else if stem.ends_with('s') && !stem.ends_with("ss") && !stem.ends_with("us") {
        stem.pop();
    }
    for (suffix, minimum) in [
        ("ation", 4),
        ("ion", 4),
        ("ment", 5),
        ("ing", 4),
        ("ed", 4),
        ("ate", 4),
        ("er", 4),
        ("or", 4),
        ("e", 4),
    ] {
        if stem.len() >= suffix.len() + minimum && stem.ends_with(suffix) {
            stem.truncate(stem.len() - suffix.len());
            let bytes = stem.as_bytes();
            let doubled = bytes.len() >= 2
                && bytes[bytes.len() - 1] == bytes[bytes.len() - 2]
                && !b"lsz".contains(&bytes[bytes.len() - 1]);
            let inflected = matches!(suffix, "ing" | "ed" | "er" | "or");
            if doubled && inflected {
                stem.pop();
            } else if inflected && stem.len() >= 6 && stem.ends_with("at") {
                stem.truncate(stem.len() - 2);
            }
            break;
        }
    }
    stem
}

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

/// How the last word of a query is completed while it is being typed.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Completion {
    None,
    /// Complete the last word anywhere it is cheap to do so.
    Typing,
}

pub(super) fn expression(terms: &[Vec<String>], flexible: bool, completion: Completion) -> String {
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
            if completion == Completion::Typing && i + 1 == terms.len() {
                options.push(format!("{}*", quoted(&choices[0])));
            }
            format!("({})", options.join(" OR "))
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

/// The words exactly as written and next to each other.
pub(super) fn literal_expression(terms: &[Vec<String>], completion: Completion) -> String {
    let words: Vec<_> = terms.iter().map(|choices| choices[0].as_str()).collect();
    let phrase = quoted(&words.join(" "));
    if completion == Completion::Typing {
        format!("{phrase} OR {phrase} *")
    } else {
        phrase
    }
}

/// Known words ordered from the rarest in the library to the most common.
/// When no article has every word, absent terms must not prevent a partial match.
pub(super) fn rarest_first(db: &Connection, terms: &[Vec<String>]) -> Result<Vec<Vec<String>>> {
    db.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS temp.doc_search_vocab USING fts5vocab(main, docs_fts, row)")?;
    let mut frequency = db.prepare_cached(
        "SELECT coalesce(sum(doc),0) FROM temp.doc_search_vocab
         WHERE term IN (SELECT value FROM json_each(?1))",
    )?;
    let mut counted = Vec::with_capacity(terms.len());
    for choices in terms {
        let spellings: BTreeSet<_> = choices.iter().flat_map(|word| forms(word)).collect();
        let count: u64 = if spellings.iter().all(|word| word.is_ascii()) {
            frequency.query_row([serde_json::to_string(&spellings)?], |row| row.get(0))?
        } else {
            // Vocabulary keys use unicode61 normalization, so `café` is stored
            // as `cafe`. Let the same tokenizer used by search count these terms.
            let expression = expression(std::slice::from_ref(choices), true, Completion::None);
            db.prepare_cached("SELECT count(*) FROM docs_fts WHERE docs_fts MATCH ?1")?
                .query_row([format!("{{title body}} : ({expression})")], |row| {
                    row.get(0)
                })?
        };
        if count > 0 {
            counted.push((count, choices.clone()));
        }
    }
    counted.sort_by_key(|(count, _)| *count);
    Ok(counted.into_iter().map(|(_, choices)| choices).collect())
}

/// Whether more headings match than are worth ranking for one common word.
pub(super) fn floods_sections(db: &Connection, expression: &str) -> Result<bool> {
    let mut probe = db.prepare_cached(
        "SELECT 1 FROM doc_sections_fts WHERE doc_sections_fts MATCH ?1 LIMIT 1 OFFSET 20000",
    )?;
    Ok(probe.exists([expression])?)
}

/// Any of the words, to find articles named for one of them.
pub(super) fn any_expression(terms: &[Vec<String>]) -> String {
    terms
        .iter()
        .flatten()
        .flat_map(|word| forms(word))
        .map(|word| quoted(&word))
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// The words next to each other.
pub(super) fn phrase_expression(terms: &[Vec<String>]) -> Option<String> {
    if !(2..=6).contains(&terms.len()) {
        return None;
    }
    let words: Vec<_> = terms.iter().map(|choices| choices[0].as_str()).collect();
    Some(quoted(&words.join(" ")))
}

/// One full-text match before the article row is read.
pub(super) struct Raw {
    pub document: i64,
    pub heading: Option<String>,
    pub anchor: Option<String>,
    pub quality: f64,
}

pub(super) fn documents(
    db: &Connection,
    scope: &DocScope,
    expression: &str,
    title_only: bool,
    window: i64,
) -> Result<Vec<Raw>> {
    let columns = if title_only { "title" } else { "{title body}" };
    let expression = format!("{columns} : ({expression})");
    // The scope is a set of rowid ranges, so matches outside it are dropped
    // before ranking and without reading any article row.
    let sql = format!(
        "SELECT docs_fts.rowid, bm25(docs_fts,12.0,1.0,0.0) AS quality FROM docs_fts
         WHERE docs_fts MATCH ?1 AND docs_fts.rowid BETWEEN ?3 AND ?4 AND {}
         ORDER BY quality, docs_fts.rowid LIMIT ?2",
        scope.contains_rowid("docs_fts.rowid")
    );
    let mut stmt = db.prepare_cached(&sql)?;
    let mut hits = Vec::new();
    // Completing a prefix merges every word that begins with it before any
    // window applies, so a prefix query is run once over everything.
    let windows = if expression.contains('*') {
        vec![(i64::MIN, i64::MAX)]
    } else {
        scope.article_windows()
    };
    for (first, last) in &windows {
        let rows = stmt.query_map(params![expression, window, first, last], |row| {
            Ok(Raw {
                document: row.get(0)?,
                heading: None,
                anchor: None,
                quality: row.get(1)?,
            })
        })?;
        for row in rows {
            hits.push(row?);
        }
    }
    if windows.len() > 1 {
        hits.sort_by(|a, b| {
            a.quality
                .total_cmp(&b.quality)
                .then_with(|| a.document.cmp(&b.document))
        });
        hits.truncate(window as usize);
    }
    Ok(hits)
}

pub(super) fn sections(
    db: &Connection,
    scope: &DocScope,
    expression: &str,
    window: i64,
) -> Result<Vec<Raw>> {
    let sql = format!(
        "SELECT s.document_rowid, s.heading, s.anchor, bm25(doc_sections_fts,2.0,1.0) AS quality,
                s.rowid
         FROM doc_sections_fts CROSS JOIN doc_sections s ON s.rowid=doc_sections_fts.rowid
         WHERE doc_sections_fts MATCH ?1 AND doc_sections_fts.rowid BETWEEN ?3 AND ?4 AND {}
         ORDER BY quality, s.rowid LIMIT ?2",
        scope.contains_rowid("s.document_rowid")
    );
    let mut stmt = db.prepare_cached(&sql)?;
    let mut hits = Vec::new();
    let windows = if expression.contains('*') {
        vec![(i64::MIN, i64::MAX)]
    } else {
        scope.section_windows(db)?
    };
    for (first, last) in &windows {
        let rows = stmt.query_map(params![expression, window, first, last], |row| {
            Ok((
                Raw {
                    document: row.get(0)?,
                    heading: Some(row.get(1)?),
                    anchor: row.get(2)?,
                    quality: row.get(3)?,
                },
                row.get::<_, i64>(4)?,
            ))
        })?;
        for row in rows {
            hits.push(row?);
        }
    }
    if windows.len() > 1 {
        hits.sort_by(|(a, a_section), (b, b_section)| {
            a.quality
                .total_cmp(&b.quality)
                .then_with(|| a_section.cmp(b_section))
        });
        hits.truncate(window as usize);
    }
    Ok(hits.into_iter().map(|(raw, _)| raw).collect())
}

/// The query as scoring sees it.
pub(super) struct Scoring<'a> {
    pub terms: &'a [Vec<String>],
    stems: Vec<Vec<String>>,
    /// The last word may still be incomplete.
    pub typing: bool,
    /// Written as a sentence or task rather than as a name.
    pub natural: bool,
    /// A flag or dotted name: its words count only as written and together.
    pub literal: bool,
    /// The dotted name in lower case, to prefer text that writes it that way.
    pub written: Option<String>,
    /// The last word is a letter or two that the index was not asked for.
    pub fragment: bool,
}

impl<'a> Scoring<'a> {
    pub fn new(terms: &'a [Vec<String>], typing: bool, natural: bool, literal: bool) -> Self {
        let stems = terms
            .iter()
            .map(|choices| choices.iter().map(|word| stem(word)).collect())
            .collect();
        Self {
            terms,
            stems,
            typing,
            natural,
            literal,
            written: None,
            fragment: false,
        }
    }

    fn matches(&self, term: usize, token: &str) -> bool {
        self.terms[term].iter().any(|choice| choice == token)
            || (!self.literal && self.stems[term].contains(&stem(token)))
    }

    fn completes(&self, term: usize, token: &str) -> bool {
        self.typing
            && term + 1 == self.terms.len()
            && token.len() > self.terms[term][0].len()
            && token.starts_with(self.terms[term][0].as_str())
    }

    fn evidence(&self, tokens: &[&str]) -> Evidence {
        let count = self.terms.len();
        let mut covered = 0;
        let mut completed = false;
        let mut unfinished = 0;
        let mut last = false;
        let mut used = HashSet::new();
        for term in 0..count {
            if let Some(index) = tokens.iter().position(|token| self.matches(term, token)) {
                covered += 1;
                last |= term + 1 == count;
                used.insert(index);
            } else if let Some((index, token)) = tokens
                .iter()
                .enumerate()
                .filter(|(_, token)| self.completes(term, token))
                .min_by_key(|(_, token)| token.len())
            {
                completed = true;
                unfinished = token.len() - self.terms[term][0].len();
                used.insert(index);
            }
        }
        let phrase = tokens.len() >= count
            && tokens.windows(count).any(|slice| {
                slice
                    .iter()
                    .enumerate()
                    .all(|(term, token)| self.matches(term, token) || self.completes(term, token))
            });
        last |= completed;
        if self.literal && !phrase {
            covered = 0;
            completed = false;
            last = false;
        }
        Evidence {
            covered,
            last,
            completed,
            unfinished,
            phrase,
            exact: phrase && tokens.len() == count,
            extra: tokens.len().saturating_sub(used.len()),
        }
    }

    /// Evidence from a title or heading, reading an identifier in it both as
    /// one word and as the words it is made of.
    fn evidence_in(&self, text: &str, keep: impl Fn(&str) -> bool) -> Evidence {
        let lower = text.to_lowercase();
        let tokens: Vec<_> = words(&lower).filter(|word| keep(word)).collect();
        let plain = self.evidence(&tokens);
        let identifiers = text
            .split_whitespace()
            .any(|word| word.contains('_') || word.chars().skip(1).any(char::is_uppercase));
        if !identifiers {
            return plain;
        }
        let expanded = super::doc_index::expand_identifiers(text).to_lowercase();
        let tokens: Vec<_> = words(&expanded).filter(|word| keep(word)).collect();
        let split = self.evidence(&tokens);
        if (split.covered, split.completed, split.phrase, split.exact)
            > (plain.covered, plain.completed, plain.phrase, plain.exact)
        {
            split
        } else {
            plain
        }
    }

    fn complete(&self, evidence: &Evidence) -> bool {
        evidence.covered + i32::from(evidence.completed) == self.terms.len() as i32
    }
}

struct Evidence {
    covered: i32,
    /// The last word was found, whole or as the start of a longer word.
    last: bool,
    completed: bool,
    /// Letters still missing from the word being typed.
    unfinished: usize,
    phrase: bool,
    exact: bool,
    extra: usize,
}

/// The name an article goes by, without a publisher prefix, a description, or
/// the path of the module that contains it.
pub(super) fn title_name(title: &str) -> &str {
    let title = title.split_once(" · ").map_or(title, |(_, topic)| topic);
    // `gap` CSS property, `FastAPI` class
    if let Some((name, _)) = title
        .strip_prefix('`')
        .and_then(|rest| rest.split_once('`'))
        .filter(|(name, _)| !name.is_empty())
    {
        return name;
    }
    let title = [" — ", " – ", " - ", " | "]
        .iter()
        .find_map(|separator| title.split_once(separator))
        .map_or(title, |(name, _)| name);
    let title = title
        .split_once(" in ")
        .filter(|(name, path)| !name.contains(' ') && (path.contains("::") || !path.contains(' ')))
        .map_or(title, |(name, _)| name);
    title
        .strip_suffix(" class")
        .unwrap_or(title)
        .trim_matches(|ch: char| ch == '`' || ch.is_whitespace())
}

/// Words that say what kind of page this is rather than what it is about.
fn title_filler(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "by"
            | "for"
            | "from"
            | "in"
            | "of"
            | "on"
            | "the"
            | "to"
            | "with"
            | "about"
            | "basics"
            | "cheatsheet"
            | "fundamentals"
            | "getting"
            | "guide"
            | "howto"
            | "intro"
            | "introduction"
            | "overview"
            | "reference"
            | "started"
            | "tutorial"
            | "using"
    )
}

pub(super) struct TextMatch {
    pub lead: Lead,
    pub heading: Option<String>,
    pub anchor: Option<String>,
    pub score: i32,
    pub quality: f64,
    /// The matching line of the article body and the excerpt that begins
    /// there, when the body was read.
    pub line: Option<(String, String)>,
    /// The article is named for one of several query words.
    pub subject: bool,
    /// The title or heading has the word that was still being typed.
    pub fragment_seen: bool,
}

impl TextMatch {
    pub fn new(lead: Lead, raw: Raw, scoring: &Scoring, publisher: &HashSet<String>) -> Self {
        let terms = scoring.terms;
        let title = lead.title.to_lowercase();
        let whole = scoring.evidence_in(&lead.title, |_| true);
        // Pack titles can prefix the actual topic with its publisher, follow a
        // name with its description, or repeat the name of the framework.
        let name = title_name(&lead.title);
        let topical = |word: &str| !title_filler(word) && !publisher.contains(word);
        let topic = scoring.evidence_in(name, topical);
        let name_lower = name.to_lowercase();
        let name_tokens: Vec<_> = words(&name_lower)
            .filter(|word| !title_filler(word) && !publisher.contains(*word))
            .collect();
        // An exact topic at the end of a contextual query, as in `python json`
        // for `json — JSON encoder and decoder`, is as good as an exact title.
        let context = terms.len().saturating_sub(name_tokens.len());
        let contextual = context > 0
            && !name_tokens.is_empty()
            && !scoring.literal
            && (title.contains(" · ")
                || terms[..context]
                    .iter()
                    .all(|choices| publisher.contains(&choices[0])))
            && name_tokens
                .iter()
                .enumerate()
                .all(|(index, token)| scoring.matches(context + index, token));
        let exact = whole.exact || (topic.exact && scoring.complete(&topic)) || contextual;
        let mut score = 45 * whole.covered
            + i32::from(whole.completed) * (30 - whole.unfinished.min(12) as i32)
            + i32::from(whole.phrase || topic.phrase || contextual) * 26
            + i32::from(exact) * 35;
        let titled = scoring.complete(&topic) || scoring.complete(&whole);
        if scoring.natural {
            // A task is more often explained in a guide than in a reference.
            let record = lead.id.split_once('/').map_or("", |(_, record)| record);
            if super::doc_index::category(&lead.pack_id, record, "") == "guide" {
                score += 8;
            }
        }
        // Named for one of the words and nothing else: the subject itself.
        let subject = !titled && topic.covered >= 1 && topic.extra == 0;
        if subject {
            score += 20;
        }
        if scoring.complete(&topic) {
            score += (20 - 4 * topic.extra as i32).max(0);
        }
        let mut fragment_seen = whole.last;
        if let Some(written) = &raw.heading {
            // An assignment containing a code example is not a named topic.
            let heading = written.to_lowercase();
            if heading != title {
                let evidence = scoring.evidence_in(written, |_| true);
                fragment_seen |= evidence.last;
                let example = heading
                    .split_once('(')
                    .is_some_and(|(prefix, _)| prefix.contains(" = "));
                let unrelated = scoring.literal
                    && (!scoring.complete(&evidence)
                        || scoring
                            .written
                            .as_ref()
                            .is_some_and(|written| !heading.contains(written.as_str())));
                // The index stems more freely than this does, and stems the
                // prefix of a word being typed. Only words found in the
                // heading count for it.
                let verified = evidence.covered + i32::from(evidence.completed);
                if (!example || evidence.exact) && !unrelated && verified > 0 {
                    let mut section = 22
                        + 20 * verified
                        + 4 * evidence.covered
                        + i32::from(evidence.phrase) * 18
                        + i32::from(evidence.exact) * 25;
                    if scoring.complete(&evidence) {
                        section += (12 - 3 * evidence.extra as i32).max(0);
                    }
                    // A sentence asks for a topic. A section headed by its
                    // words answers it. A signature that happens to contain
                    // them is a much weaker answer.
                    let signature =
                        heading.contains(['(', '=', '<', '_']) || heading.contains("::");
                    if scoring.natural && signature && !evidence.exact {
                        section /= 2;
                    } else if scoring.natural && scoring.complete(&evidence) && evidence.extra <= 2
                    {
                        section += 20;
                    }
                    // A title that already names the topic is the evidence. Its
                    // sections add a place to land, not a second vote, so that
                    // the article about a topic leads one with a chapter on it.
                    score += if titled { section / 2 } else { section };
                }
            }
        }
        Self {
            lead,
            heading: raw.heading,
            anchor: raw.anchor,
            score,
            quality: raw.quality,
            line: None,
            subject,
            fragment_seen,
        }
    }

    /// Adds what the article body says about the query, and remembers the line
    /// that says it so the result can show why it matched.
    pub fn read_body(&mut self, body: &str, scoring: &Scoring) {
        if scoring.fragment && !scoring.literal && !self.fragment_seen {
            // A word in the text begins with the letters typed so far.
            let fragment = scoring.terms[scoring.terms.len() - 1][0].as_str();
            let folded = body.to_ascii_lowercase();
            self.fragment_seen = folded.match_indices(fragment).any(|(start, _)| {
                !folded[..start]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_alphanumeric)
            });
        }
        let Some((offset, line, evidence, completes)) = best_line(body, scoring) else {
            return;
        };
        // For a name, the letters must continue the name where it is written.
        self.fragment_seen |= scoring.literal && completes;
        if self.heading.is_none() {
            self.score += evidence;
        }
        let excerpt = super::compact_excerpt(&body[offset..]);
        self.line = Some((line, excerpt));
    }

    pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
        b.score
            .cmp(&a.score)
            .then_with(|| a.quality.total_cmp(&b.quality))
            .then_with(|| a.lead.id.cmp(&b.lead.id))
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
            if named.contains(&hit.lead.id) {
                hit.score += 35;
            }
        }
        Ok(())
    }
}

/// Finds the line of an article that best answers the query: the one with the
/// most query words, preferring words that appear together and in order.
fn best_line(body: &str, scoring: &Scoring) -> Option<(usize, String, i32, bool)> {
    const SCANNED: usize = 400_000;
    let body = &body[..body.floor_char_boundary(body.len().min(SCANNED))];
    // ASCII folding keeps every byte offset, so lines of the folded text
    // line up with lines of the original.
    let folded = body.to_ascii_lowercase();
    let stubs: Vec<_> = scoring
        .terms
        .iter()
        .flatten()
        .map(|word| &word[..word.floor_char_boundary(word.len().min(4))])
        .collect();
    let mut best: Option<(i32, usize, &str, bool)> = None;
    let mut offset = 0;
    for raw in folded.split_inclusive('\n') {
        let start = offset + raw.len() - raw.trim_start().len();
        offset += raw.len();
        let lower = raw.trim();
        if lower.len() < 3 {
            continue;
        }
        // Most lines mention none of the words. Skip them before tokenizing.
        if !stubs.iter().any(|stub| lower.contains(stub)) {
            continue;
        }
        let line = &body[start..start + lower.len()];
        let tokens: Vec<_> = words(lower).collect();
        let evidence = scoring.evidence(&tokens);
        if evidence.covered == 0 && !evidence.completed {
            continue;
        }
        let pairs = (0..scoring.terms.len().saturating_sub(1))
            .filter(|&term| {
                tokens.windows(2).any(|pair| {
                    scoring.matches(term, pair[0]) && scoring.matches(term + 1, pair[1])
                })
            })
            .count() as i32;
        let mut score = evidence.covered * 3
            + i32::from(scoring.fragment && evidence.last) * 5
            + i32::from(evidence.completed) * 2
            + pairs * 12
            + i32::from(evidence.phrase) * 15;
        if scoring
            .written
            .as_ref()
            .is_some_and(|written| lower.contains(written.as_str()))
        {
            score += 10;
        }
        // Prefer a line that can be shown whole.
        if line.chars().count() > 180 {
            score -= 1;
        }
        if best.is_none_or(|(old, _, _, _)| score > old) {
            best = Some((score, start, line, evidence.last));
        }
    }
    let (score, start, line, last) = best?;
    let (skipped, shown) = excerpt_around(line, scoring);
    Some((start + skipped, shown, score, last))
}

/// A long paragraph is shown from just before its first matching word.
fn excerpt_around(line: &str, scoring: &Scoring) -> (usize, String) {
    const SHOWN: usize = 180;
    if line.chars().count() <= SHOWN {
        return (0, line.to_owned());
    }
    let lower = line.to_lowercase();
    let first = scoring
        .terms
        .iter()
        .flatten()
        .filter_map(|word| lower.find(word.as_str()))
        .min()
        .filter(|_| lower.len() == line.len())
        .unwrap_or(0);
    let mut start = line.floor_char_boundary(first.saturating_sub(40));
    if start > 0 {
        // Begin at a word.
        start = line[start..]
            .find(' ')
            .map_or(start, |space| start + space + 1);
    }
    (start, line[start..].chars().take(SHOWN).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_join_the_forms_of_a_topic() {
        for (left, right) in [
            ("websockets", "websocket"),
            ("validation", "validate"),
            ("validations", "validating"),
            ("migration", "migrate"),
            ("deployment", "deploying"),
            ("configuration", "configuring"),
            ("logging", "logged"),
            ("caching", "cache"),
            ("dependencies", "dependency"),
            ("routes", "routing"),
            ("formatter", "format"),
            ("extractors", "extract"),
            ("iterator", "iterate"),
        ] {
            assert_eq!(stem(left), stem(right), "{left} {right}");
        }
        for word in ["string", "class", "status", "file", "type", "test"] {
            assert_eq!(stem(word), word);
        }
    }

    #[test]
    fn names_drop_descriptions_and_containers() {
        for (title, name) in [
            ("pathlib — Object-oriented filesystem paths", "pathlib"),
            ("HashMap in std::collections", "HashMap"),
            ("Spring Boot · Logging", "Logging"),
            ("cargo build - The Cargo Book", "cargo build"),
            ("`FastAPI` class", "FastAPI"),
            ("`gap` CSS property", "gap"),
            ("Routing in ASP.NET Core", "Routing in ASP.NET Core"),
            ("State: A Component's Memory", "State: A Component's Memory"),
        ] {
            assert_eq!(title_name(title), name);
        }
    }

    #[test]
    fn two_slips_are_still_the_same_word() {
        assert!(within_two_edits("middlewear", "middleware"));
        assert!(within_two_edits("subprocses", "subprocess"));
        assert!(!within_two_edits("middleware", "middleman"));
    }

    #[test]
    fn spellings_without_separators_are_recovered() {
        assert!(separated_symbols("readtext").contains(&"read_text".to_owned()));
        assert!(separated_symbols("readtostring").contains(&"read_to_string".to_owned()));
        assert!(joined_symbols("read text").contains(&"read_text".to_owned()));
        assert!(separated_symbols("Read").is_empty());
    }

    #[test]
    fn excerpts_show_the_line_that_matched() {
        let terms = vec![vec!["read".to_owned()], vec!["file".to_owned()]];
        let scoring = Scoring::new(&terms, false, true, false);
        let body = "Introduction\n\nFiles are everywhere.\n\nTo read a file, call open().\n\nThe file can be read later.";
        let (offset, line, _, _) = best_line(body, &scoring).unwrap();
        assert_eq!(line, "To read a file, call open().");
        assert!(body[offset..].starts_with("To read a file"));
    }
}
