//! DevDocs acquisition for the desktop job worker and CLI.

use crate::{
    devdocs_html::{self, MAX_IMAGE_BYTES},
    doc_library, tasks, updates, Catalog, Context, DocPack,
};
use anyhow::{bail, Context as _, Result};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

const MANIFEST_URL: &str = "https://devdocs.io/docs.json";
const ARCHIVE_BASE: &str = "https://downloads.devdocs.io/";
const MAX_MANIFEST: u64 = 16 * 1024 * 1024;
const MAX_METADATA: u64 = 64 * 1024 * 1024;
const MAX_DATABASE: u64 = 512 * 1024 * 1024;
const MAX_ARCHIVE: u64 = 1024 * 1024 * 1024;
const MAX_IMAGE_TOTAL: u64 = 32 * 1024 * 1024;
const SHARD_BYTES: usize = 64 * 1024 * 1024;

const PAGE_ENCODING: &percent_encoding::AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'/');

#[derive(Clone, Default, Deserialize)]
struct Metadata {
    slug: Option<String>,
    name: Option<String>,
    release: Option<Value>,
    version: Option<Value>,
    mtime: Option<i64>,
    db_size: Option<u64>,
    attribution: Option<String>,
    links: Option<PublisherLinks>,
}

#[derive(Clone, Deserialize)]
struct PublisherLinks {
    home: Option<String>,
}

impl Metadata {
    fn merge(&mut self, snapshot: Self) {
        self.slug = snapshot.slug.or(self.slug.take());
        self.name = snapshot.name.or(self.name.take());
        self.release = snapshot.release.or(self.release.take());
        self.version = snapshot.version.or(self.version.take());
        self.mtime = snapshot.mtime.or(self.mtime);
        self.db_size = snapshot.db_size.or(self.db_size);
        self.attribution = snapshot.attribution.or(self.attribution.take());
        self.links = snapshot.links.or(self.links.take());
    }
}

fn display_version(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) if !value.is_empty() => Some(value.to_owned()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn valid_slug(slug: &str) -> bool {
    fn component(value: &str) -> bool {
        value
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'_' | b'.')
            })
    }
    slug.len() <= 128
        && slug.split_once('~').map_or_else(
            || component(slug),
            |(family, edition)| component(family) && component(edition),
        )
}

fn family(slug: &str) -> &str {
    slug.split_once('~').map_or(slug, |(family, _)| family)
}

fn pack_id(slug: &str) -> String {
    format!("devdocs-{}", family(slug).replace('.', "_"))
}

fn numeric_version(value: &str) -> Option<Vec<u64>> {
    let value = value
        .trim()
        .strip_prefix(['v', 'V'])
        .unwrap_or(value.trim());
    let end = value
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(value.len());
    let mut parts = value[..end]
        .trim_end_matches('.')
        .split('.')
        .map(str::parse)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    while parts.len() > 1 && parts.last() == Some(&0) {
        parts.pop();
    }
    Some(parts)
}

fn snapshot_order<'a>(
    slug: &'a str,
    meta: Option<&Metadata>,
) -> (bool, Option<Vec<u64>>, bool, i64, &'a str) {
    let edition = slug.split_once('~').map(|(_, edition)| edition);
    let canonical = edition.is_none();
    let version = meta
        .and_then(|row| {
            display_version(row.release.as_ref())
                .and_then(|v| numeric_version(&v))
                .or_else(|| display_version(row.version.as_ref()).and_then(|v| numeric_version(&v)))
        })
        .or_else(|| edition.and_then(numeric_version));
    (
        canonical && version.is_none(),
        version,
        canonical,
        meta.and_then(|row| row.mtime).unwrap_or(0),
        slug,
    )
}

fn fetch(
    url: &str,
    destination: &Path,
    limit: u64,
    job: Option<&mut tasks::Job>,
    label: &str,
) -> Result<()> {
    let mut command = Command::new("/usr/bin/curl");
    command
        .args([
            "--disable",
            "--fail",
            "--location",
            "--max-redirs",
            "5",
            "--silent",
            "--show-error",
            "--globoff",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--connect-timeout",
            "12",
            "--max-time",
            if limit == MAX_MANIFEST { "30" } else { "600" },
            "--user-agent",
            "Mozilla/5.0",
            "--max-filesize",
        ])
        .arg(limit.to_string())
        .arg("--output")
        .arg(destination)
        .arg("--url")
        .arg(url);
    if let Some(job) = job {
        job.command(&mut command, label)?;
    } else {
        tasks::normalize_child_signals(&mut command);
        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .with_context(|| label.to_owned())?;
        if !status.success() {
            bail!("{label} failed. Check your connection and retry.");
        }
    }
    if fs::metadata(destination)?.len() > limit {
        bail!("DevDocs download exceeds its size limit");
    }
    Ok(())
}

fn bounded_json<T: DeserializeOwned>(input: impl Read, name: &str, limit: u64) -> Result<T> {
    let mut data = Vec::new();
    input.take(limit + 1).read_to_end(&mut data)?;
    if data.len() as u64 > limit {
        bail!("{name} exceeds its input size limit");
    }
    serde_json::from_slice(data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&data))
        .with_context(|| format!("Read DevDocs {name}"))
}

fn json_file<T: DeserializeOwned>(path: &Path, limit: u64) -> Result<T> {
    let file = File::open(path).with_context(|| format!("Open {}", path.display()))?;
    if !file.metadata()?.is_file() {
        bail!("Expected a regular DevDocs input file");
    }
    bounded_json(file, &path.display().to_string(), limit)
}

type Manifest = BTreeMap<String, Metadata>;

fn manifest_rows(rows: Vec<Metadata>) -> Result<Manifest> {
    let mut manifest = Manifest::new();
    for row in rows {
        let slug = row
            .slug
            .as_deref()
            .filter(|slug| valid_slug(slug))
            .context("Invalid DevDocs catalog slug")?;
        if manifest
            .get(slug)
            .is_none_or(|previous| row.mtime.unwrap_or(0) > previous.mtime.unwrap_or(0))
        {
            manifest.insert(slug.into(), row);
        }
    }
    Ok(manifest)
}

fn fetch_manifest(destination: &Path, job: Option<&mut tasks::Job>) -> Result<Manifest> {
    let path = destination.join("docs.json");
    fetch(
        MANIFEST_URL,
        &path,
        MAX_MANIFEST,
        job,
        "Checking DevDocs catalog",
    )?;
    manifest_rows(json_file(&path, MAX_MANIFEST)?)
}

pub(super) fn catalog(request: &Value) -> Result<Value> {
    if request["offline"] == true {
        bail!("Checking the DevDocs catalog requires a connection.");
    }
    let stage = tempfile::Builder::new()
        .prefix("devline-devdocs-")
        .tempdir()?;
    let manifest = fetch_manifest(stage.path(), None)?;
    let name = |slug: &str, row: &Metadata| row.name.clone().unwrap_or_else(|| family(slug).into());
    let mut rows = manifest.iter().collect::<Vec<_>>();
    rows.sort_by(|(a_slug, a), (b_slug, b)| {
        name(a_slug, a)
            .to_lowercase()
            .cmp(&name(b_slug, b).to_lowercase())
            .then_with(|| family(a_slug).cmp(family(b_slug)))
            .then_with(|| snapshot_order(b_slug, Some(b)).cmp(&snapshot_order(a_slug, Some(a))))
    });
    let docsets = rows.into_iter().map(|(slug, row)| {
        json!({
            "slug": slug,
            "name": name(slug, row),
            "release": display_version(row.release.as_ref()),
            "version": display_version(row.version.as_ref()),
            "mtime": row.mtime,
            "db_size": row.db_size,
            "source_url": row.links.as_ref().and_then(|links| devdocs_html::web_source(links.home.as_deref()?)),
        })
    }).collect::<Vec<_>>();
    Ok(json!({"docsets": docsets}))
}

fn local_inputs(source: &Path) -> Result<(Manifest, BTreeMap<String, PathBuf>)> {
    fn name(path: &Path) -> Option<&str> {
        path.file_name()?.to_str()
    }
    if source.is_file() {
        let slug = name(source)
            .and_then(|name| name.strip_suffix(".tar.gz"))
            .context("Choose a DevDocs .tar.gz archive or a directory")?;
        return Ok((Manifest::new(), [(slug.into(), source.into())].into()));
    }
    if !source.is_dir() {
        bail!("Choose a DevDocs checkout, docset directory, or .tar.gz archive");
    }
    let public_docs = source.join("public/docs");
    let source = if public_docs.is_dir() {
        public_docs.as_path()
    } else {
        source
    };
    if source.join("db.json").is_file() {
        let slug = name(source).context("Invalid DevDocs docset directory name")?;
        return Ok((Manifest::new(), [(slug.into(), source.into())].into()));
    }

    let manifest_path = source.join("docs.json");
    let manifest = if manifest_path.is_file() {
        manifest_rows(json_file(&manifest_path, MAX_MANIFEST)?)?
    } else {
        Manifest::new()
    };
    let mut paths = BTreeMap::new();
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let Some(name) = name(&path) else { continue };
        if valid_slug(name) && path.is_dir() && path.join("db.json").is_file() {
            paths.insert(name.into(), path);
        } else if let Some(slug) = name.strip_suffix(".tar.gz").filter(|slug| valid_slug(slug)) {
            if path.is_file() {
                paths.entry(slug.into()).or_insert(path);
            }
        }
    }
    Ok((manifest, paths))
}

fn select_slugs(
    available: Vec<String>,
    requested: Vec<String>,
    latest: bool,
    metadata: &Manifest,
) -> Result<Vec<String>> {
    if requested.is_empty() && !latest {
        return Ok(available);
    }
    let mut families = BTreeMap::<&str, Vec<&str>>::new();
    for slug in &available {
        families.entry(family(slug)).or_default().push(slug);
    }
    let requests = if requested.is_empty() {
        families.keys().copied().collect::<Vec<_>>()
    } else {
        requested.iter().map(String::as_str).collect()
    };
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for slug in requests {
        let chosen = if available.iter().any(|item| item == slug) && (!latest || slug.contains('~'))
        {
            vec![slug]
        } else {
            let candidates = families
                .get(slug)
                .with_context(|| format!("DevDocs docset {slug:?} was not found"))?;
            if latest {
                let newest = candidates
                    .iter()
                    .copied()
                    .max_by_key(|slug| snapshot_order(slug, metadata.get(*slug)))
                    .context("DevDocs family has no available snapshots")?;
                vec![newest]
            } else {
                candidates.clone()
            }
        };
        for candidate in chosen {
            if seen.insert(candidate) {
                selected.push(candidate.to_owned());
            }
        }
    }
    Ok(selected)
}

#[derive(Deserialize)]
struct Index {
    #[serde(default)]
    entries: Vec<IndexEntry>,
}

#[derive(Deserialize)]
struct IndexEntry {
    name: Option<String>,
    path: Option<String>,
    #[serde(rename = "type")]
    group: Option<String>,
}

type Pages = BTreeMap<String, String>;

enum Assets {
    Archive(BTreeMap<PathBuf, Vec<u8>>),
    Directory(PathBuf),
}

impl Assets {
    fn get(&self, key: &str, raw: &str) -> Option<Vec<u8>> {
        let link = fluent_uri::IriRef::parse(raw).ok()?;
        if link.scheme().is_some() || link.authority().is_some() {
            return None;
        }
        let path = link.path().decode().to_string().ok()?;
        if path.contains(['\\', '\0']) || !is_image(Path::new(path.as_ref())) {
            return None;
        }
        let relative = local_path(key, &path)?;
        match self {
            Assets::Archive(images) => images.get(Path::new(&relative)).cloned(),
            Assets::Directory(root) => {
                let path = root.join(relative).canonicalize().ok()?;
                let meta = path.metadata().ok()?;
                if !path.starts_with(root) || !meta.is_file() || meta.len() > MAX_IMAGE_BYTES {
                    return None;
                }
                fs::read(path).ok()
            }
        }
    }
}

struct Snapshot {
    meta: Metadata,
    index: Index,
    pages: Pages,
    assets: Assets,
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|suffix| {
            matches!(
                suffix.to_ascii_lowercase().as_str(),
                "png" | "jpeg" | "jpg" | "gif" | "svg" | "webp"
            )
        })
}

fn read_snapshot(path: &Path, job: Option<&tasks::Job>) -> Result<Snapshot> {
    if path.is_dir() {
        let meta = path.join("meta.json");
        return Ok(Snapshot {
            meta: if meta.is_file() {
                json_file(&meta, MAX_METADATA)?
            } else {
                Metadata::default()
            },
            index: json_file(&path.join("index.json"), MAX_METADATA)?,
            pages: json_file(&path.join("db.json"), MAX_DATABASE)?,
            assets: Assets::Directory(path.canonicalize()?),
        });
    }

    let file = File::open(path).with_context(|| format!("Open {}", path.display()))?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_ARCHIVE {
        bail!("DevDocs archive is not a regular file within the size limit");
    }
    let mut archive = tar::Archive::new(GzDecoder::new(file).take(2 * MAX_ARCHIVE));
    let mut pages = None;
    let mut index = None;
    let mut meta = None;
    let mut images = BTreeMap::new();
    let mut image_size = 0;
    for entry in archive.entries()?.raw(true) {
        checkpoint(job)?;
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let name = entry
            .path()?
            .components()
            .filter(|component| *component != Component::CurDir)
            .collect::<PathBuf>();
        if name.as_os_str().is_empty()
            || name
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            continue;
        }
        let size = entry.size();
        let file = name.to_str();
        match file {
            Some("meta.json") if meta.is_none() => {
                meta = Some(bounded_json(entry, "meta.json", MAX_METADATA)?)
            }
            Some("db.json") if pages.is_none() => {
                pages = Some(bounded_json(entry, "db.json", MAX_DATABASE)?)
            }
            Some("index.json") if index.is_none() => {
                index = Some(bounded_json(entry, "index.json", MAX_METADATA)?)
            }
            Some("meta.json" | "db.json" | "index.json") => {
                bail!("Duplicate DevDocs archive input {name:?}")
            }
            _ if size <= MAX_IMAGE_BYTES
                && image_size + size <= MAX_IMAGE_TOTAL
                && is_image(&name)
                && !images.contains_key(&name) =>
            {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                image_size += size;
                images.insert(name, bytes);
            }
            _ => {}
        }
    }
    Ok(Snapshot {
        meta: meta.unwrap_or_default(),
        index: index.context("DevDocs archive must contain index.json at its root")?,
        pages: pages.context("DevDocs archive must contain db.json at its root")?,
        assets: Assets::Archive(images),
    })
}

fn checkpoint(job: Option<&tasks::Job>) -> Result<()> {
    job.map_or(Ok(()), tasks::Job::checkpoint)
}

fn local_path(key: &str, path: &str) -> Option<String> {
    let mut segments = Vec::new();
    if let Some(parent) = key.rsplit_once('/').map(|(parent, _)| parent) {
        if !path.starts_with('/') {
            segments.extend(parent.split('/'));
        }
    }
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            _ => segments.push(segment),
        }
    }
    Some(segments.join("/"))
}

fn page_key<'a>(path: &str, sources: &'a BTreeMap<String, String>) -> Option<&'a str> {
    let path = path.trim_start_matches('/');
    let path = if matches!(path, "" | "." | "./") {
        "index".to_owned()
    } else if path.ends_with('/') {
        format!("{path}index")
    } else {
        path.to_owned()
    };
    let path = path.strip_suffix(".html").unwrap_or(&path);
    let path = path.strip_suffix(".md").unwrap_or(path);
    let decoded = percent_decode_str(path).decode_utf8().ok();
    let lookup = |candidate: &str| {
        sources
            .get_key_value(candidate)
            .map(|(key, _)| key.as_str())
    };
    lookup(path)
        .or_else(|| decoded.as_deref().and_then(lookup))
        .or_else(|| lookup(&format!("{path}/index")))
}

fn offline_link(raw: &str, key: &str, sources: &BTreeMap<String, String>) -> Option<String> {
    if raw.starts_with('#') || raw.is_empty() || raw.chars().any(|c| c <= ' ' || c == '\\') {
        return None;
    }
    let link = fluent_uri::IriRef::parse(raw).ok()?;
    if link.scheme().is_some() || link.authority().is_some() || raw.starts_with("//") {
        return None;
    }
    let path = link.path().as_str();
    let target = if path.is_empty() {
        key
    } else {
        let mut relative = local_path(key, path)?;
        if path.ends_with('/') {
            relative.push('/');
        }
        page_key(&relative, sources)?
    };
    let source = sources.get(target)?;
    Some(link.fragment().map_or_else(
        || source.clone(),
        |fragment| format!("{source}#{}", fragment.as_str()),
    ))
}

#[derive(Default)]
struct EntryAliases {
    entries: BTreeMap<String, Option<String>>,
    title: Option<String>,
    group: Option<String>,
    group_for_page: bool,
}

#[derive(Serialize)]
struct Article {
    id: String,
    title: String,
    body: String,
    html: String,
    source_url: String,
    aliases: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    alias_anchors: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
}

fn alias_index(index: Index, sources: &BTreeMap<String, String>) -> BTreeMap<String, EntryAliases> {
    let mut aliases = BTreeMap::<String, EntryAliases>::new();
    for entry in index.entries {
        let (Some(name), Some(path)) = (entry.name, entry.path) else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() {
            continue;
        }
        let (path, anchor) =
            path.split_once('#')
                .map_or((path.as_str(), None), |(path, anchor)| {
                    (
                        path,
                        Some(percent_decode_str(anchor).decode_utf8_lossy().into_owned()),
                    )
                });
        let key = page_key(path, sources)
            .or_else(|| page_key(path.split('?').next().unwrap_or(path), sources));
        let Some(key) = key else { continue };
        let row = aliases.entry(key.into()).or_default();
        if anchor.is_none() && row.title.is_none() {
            row.title = Some(name.into());
        }
        if let Some(group) = entry
            .group
            .filter(|name| !name.trim().is_empty() && name.len() <= 256)
        {
            if row.group.is_none() || (!row.group_for_page && anchor.is_none()) {
                row.group = Some(group.trim().into());
                row.group_for_page = anchor.is_none();
            }
        }
        row.entries.insert(name.into(), anchor);
    }
    aliases
}

fn shard_file(directory: &Path, number: usize) -> Result<BufWriter<File>> {
    let mut file = BufWriter::new(File::create(
        directory.join(doc_library::shard_name(number)),
    )?);
    file.write_all(b"[\n")?;
    Ok(file)
}

fn prepare_shards(
    snapshot: Snapshot,
    directory: &Path,
    base: &str,
    job: &mut Option<&mut tasks::Job>,
) -> Result<(usize, bool)> {
    if snapshot.pages.is_empty() {
        bail!("DevDocs page database is empty");
    }
    let mut sources = BTreeMap::new();
    for (key, raw) in &snapshot.pages {
        checkpoint(job.as_deref())?;
        if key.is_empty()
            || key.len() > 4096
            || key.starts_with('/')
            || key.split('/').any(|part| part == "..")
            || key.chars().any(|c| c <= '\u{1f}' || c == '\\')
        {
            bail!("Invalid DevDocs page key: {key:?}");
        }
        let source = devdocs_html::publisher_url(raw)
            .unwrap_or_else(|| format!("{base}{}", utf8_percent_encode(key, PAGE_ENCODING)));
        sources.insert(key.clone(), source);
    }
    let mut aliases = alias_index(snapshot.index, &sources);
    let mut shard_number = 1;
    let mut size = 4;
    let mut shard = shard_file(directory, shard_number)?;
    let has_entry = snapshot.pages.contains_key("index");
    if let Some(job) = job.as_deref_mut() {
        job.stage("Preparing offline documentation")?;
    }
    for (key, raw) in snapshot.pages {
        checkpoint(job.as_deref())?;
        let source = &sources[&key];
        let article = devdocs_html::article(
            &raw,
            source,
            |href| offline_link(href, &key, &sources),
            |path| snapshot.assets.get(&key, path),
        )
        .with_context(|| format!("Prepare DevDocs article {key:?}"))?;
        let alias = aliases.remove(&key).unwrap_or_default();
        let title = if article.title.is_empty() {
            alias.title.unwrap_or_else(|| key.clone())
        } else {
            article.title
        };
        let encoded = serde_json::to_vec(&Article {
            id: key.clone(),
            title,
            body: article.text,
            html: article.html,
            source_url: source.clone(),
            aliases: alias.entries.keys().cloned().collect(),
            alias_anchors: alias
                .entries
                .into_iter()
                .filter_map(|(name, anchor)| {
                    anchor
                        .filter(|anchor| article.anchors.contains(anchor))
                        .map(|anchor| (name, anchor))
                })
                .collect(),
            group: alias.group,
        })?;
        if encoded.len() + 4 > SHARD_BYTES {
            bail!("DevDocs article exceeds the size limit: {key}");
        }
        if size > 4 && size + encoded.len() + 2 > SHARD_BYTES {
            shard.write_all(b"\n]\n")?;
            shard.flush()?;
            shard_number += 1;
            shard = shard_file(directory, shard_number)?;
            size = 4;
        }
        if size > 4 {
            shard.write_all(b",\n")?;
        }
        shard.write_all(&encoded)?;
        size += encoded.len() + 2;
    }
    shard.write_all(b"\n]\n")?;
    shard.flush()?;
    Ok((shard_number, has_entry))
}

fn release_hash(directory: &Path, shards: usize, job: Option<&tasks::Job>) -> Result<String> {
    let mut names = (1..=shards)
        .map(doc_library::shard_name)
        .collect::<Vec<_>>();
    names.push("SOURCES.md".into());
    names.sort();
    let mut digest = Sha256::new();
    for name in names {
        checkpoint(job)?;
        digest.update(name.as_bytes());
        let mut file = File::open(directory.join(name))?;
        digest.update(file.metadata()?.len().to_be_bytes());
        io::copy(&mut file, &mut digest)?;
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn import_snapshot(
    ctx: &Context,
    workspace: &Path,
    path: &Path,
    selected: &str,
    manifest: Option<&Metadata>,
    keep_latest: bool,
    job: &mut Option<&mut tasks::Job>,
) -> Result<Value> {
    if let Some(job) = job.as_deref_mut() {
        job.stage("Reading DevDocs snapshot")?;
    }
    let mut snapshot = read_snapshot(path, job.as_deref())?;
    let mut meta = manifest.cloned().unwrap_or_default();
    meta.merge(std::mem::take(&mut snapshot.meta));
    let slug = meta.slug.as_deref().unwrap_or(selected);
    if !valid_slug(slug) || (manifest.is_some() && selected != slug) {
        bail!("DevDocs snapshot metadata does not match its catalog slug");
    }
    let id = pack_id(slug);
    if !doc_library::valid_doc_id(&id) {
        bail!("DevDocs docset slug is too long");
    }
    let base = format!("https://devdocs.io/{slug}/");
    let name = meta
        .name
        .as_deref()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| family(slug));
    let source_release = display_version(meta.release.as_ref())
        .or_else(|| display_version(meta.version.as_ref()))
        .or_else(|| {
            slug.split_once('~')
                .map(|(_, edition)| edition)
                .filter(|edition| numeric_version(edition).is_some())
                .map(str::to_owned)
        });
    let release = source_release.as_deref().unwrap_or("snapshot");
    let sanitized = release
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    let sanitized = sanitized.trim_matches(['.', '_', '-']);
    let release_key = if sanitized.is_empty() {
        "snapshot"
    } else {
        &sanitized[..sanitized.len().min(40)]
    };
    let directory = workspace.join("pack");
    fs::create_dir(&directory)?;
    let (shards, has_entry) = prepare_shards(snapshot, &directory, &base, job)?;
    let notice = meta
        .attribution
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(devdocs_html::attribution)
        .transpose()?
        .unwrap_or_else(|| {
            "See the attribution supplied with each article where available.".into()
        });
    fs::write(directory.join("SOURCES.md"), format!(
        "# {name}\n\nPersonal offline snapshot generated from [DevDocs]({base}).\n\nCatalog slug: `{slug}`\n\nSource release: `{release}`\n\nEach article retains publisher attribution where supplied by the snapshot. Devline sanitizes markup and rewrites links between included articles.\n\n{notice}\n"
    ))?;
    let version = format!(
        "{release_key}+devdocs.{}",
        release_hash(&directory, shards, job.as_deref())?
    );
    let relative = format!("docs/{id}/{version}");
    let destination = workspace.join(&relative);
    fs::create_dir_all(destination.parent().context("Invalid DevDocs pack path")?)?;
    fs::rename(&directory, &destination)?;
    let snapshot_date = meta
        .mtime
        .filter(|time| (0..=253402300799).contains(time))
        .and_then(|time| DateTime::<Utc>::from_timestamp(time, 0))
        .map(|time| time.date_naive().to_string());
    let pack: DocPack = serde_json::from_value(json!({
        "id": id, "name": format!("DevDocs · {name}"), "stack": id, "version": version,
        "description": format!("Personal DevDocs snapshot of {name}"), "source_url": base,
        "license": "See the original publisher notices in this snapshot", "path": relative,
        "local_only": true, "shards": shards, "upstream_version": source_release,
        "snapshot_date": snapshot_date, "entry_id": has_entry.then_some("index"),
        "coverage": ["Articles and symbol index provided by this DevDocs snapshot"],
        "omissions": ["Remote assets, interactive content and pages absent from DevDocs"],
    }))?;
    let source = Context {
        home: ctx.home.clone(),
        catalog_dir: workspace.to_owned(),
        catalog: Catalog {
            schema_version: 1,
            docs: vec![pack],
            templates: vec![],
        },
    };
    if let Some(job) = job.as_deref_mut() {
        job.stage("Indexing offline documentation")?;
    }
    let mut result = doc_library::import_local_with_checkpoint(
        &source,
        &id,
        &version,
        keep_latest,
        &mut || checkpoint(job.as_deref()),
    )?;
    result["slug"] = json!(slug);
    Ok(result)
}

pub(super) fn import(
    ctx: &Context,
    request: &Value,
    mut job: Option<&mut tasks::Job>,
) -> Result<Value> {
    let source = request["path"].as_str().map(Path::new);
    let online = request["online"].as_bool().unwrap_or(source.is_none());
    if online == source.is_some() {
        bail!("Choose either a local DevDocs source or an online import.");
    }
    if online && request["offline"] == true {
        bail!("Downloading DevDocs requires a connection. Pass a local source in offline mode.");
    }
    let stage = tempfile::Builder::new()
        .prefix("devline-devdocs-")
        .tempdir()?;
    let (manifest, paths) = if let Some(source) = source {
        local_inputs(source)?
    } else {
        (
            fetch_manifest(stage.path(), job.as_deref_mut())?,
            BTreeMap::new(),
        )
    };
    let available: Vec<_> = if online {
        manifest.keys().cloned().collect()
    } else {
        paths.keys().cloned().collect()
    };
    let latest = request["keep_latest"] == true || updates::keep_latest(ctx)?;
    let exact = request["id"].as_str();
    let selected = if let Some(slug) = exact {
        if !valid_slug(slug) || !available.iter().any(|candidate| candidate == slug) {
            bail!("DevDocs docset {slug:?} was not found");
        }
        vec![slug.to_owned()]
    } else {
        let requested = request["docs"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .context("Invalid DevDocs docset selection")
            })
            .collect::<Result<Vec<_>>>()?;
        let mut metadata = manifest.clone();
        if latest {
            for (slug, path) in &paths {
                if path.is_dir() {
                    if let Ok(row) = json_file::<Metadata>(&path.join("meta.json"), 1024 * 1024) {
                        if row
                            .slug
                            .as_deref()
                            .is_none_or(|candidate| candidate == slug)
                        {
                            metadata.entry(slug.clone()).or_default().merge(row);
                        }
                    }
                }
            }
        }
        select_slugs(available, requested, latest, &metadata)?
    };
    if selected.is_empty() {
        bail!("No DevDocs snapshots found");
    }

    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();
    for slug in selected {
        checkpoint(job.as_deref())?;
        let workspace = tempfile::Builder::new()
            .prefix("snapshot-")
            .tempdir_in(stage.path())?;
        let attempt = (|| {
            let path = if online {
                let path = workspace.path().join("download.tar.gz");
                let label = format!(
                    "Downloading {}",
                    manifest[&slug].name.as_deref().unwrap_or(&slug)
                );
                fetch(
                    &format!("{ARCHIVE_BASE}{slug}.tar.gz"),
                    &path,
                    MAX_ARCHIVE,
                    job.as_deref_mut(),
                    &label,
                )?;
                path
            } else {
                paths[&slug].clone()
            };
            import_snapshot(
                ctx,
                workspace.path(),
                &path,
                &slug,
                manifest.get(&slug),
                latest,
                &mut job,
            )
        })();
        match attempt {
            Ok(item) if item["skipped"] == true => skipped.push(item),
            Ok(item) => installed.push(item),
            Err(error) => {
                checkpoint(job.as_deref())?;
                let message = format!("{error:#}");
                if let Some(job) = job.as_deref() {
                    writeln!(job.log_writer()?, "{slug}: {message}")?;
                }
                failed.push(json!({"slug": slug, "error": message}));
            }
        }
    }
    Ok(json!({"installed": installed, "skipped": skipped, "failed": failed}))
}
