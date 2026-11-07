use crate::{install_doc_pack, lock_library, updates::compare_doc_releases, Context, DocPack};
use anyhow::{bail, Context as _, Result};
use flate2::read::GzDecoder;
use fluent_uri::Iri;
use fs2::FileExt;
use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const DEFAULT_CHANNEL: &str =
    "https://github.com/fpl0/devline/releases/download/docsets-v1/docs-manifest.json";
const MAX_MANIFEST: u64 = 2 * 1024 * 1024;
const MAX_LOCAL_METADATA: u64 = MAX_MANIFEST + 1024 * 1024;
const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;
const MAX_UNPACKED: u64 = 1024 * 1024 * 1024;
const MAX_FILE: u64 = 128 * 1024 * 1024;
const MAX_FILES: usize = 10_000;
const RECORD: &str = ".devline-pack.json";
static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Release {
    pub pack: DocPack,
    pub release_notes: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Manifest {
    schema_version: u32,
    releases: Vec<Release>,
}

#[derive(Deserialize, Serialize)]
struct Feed {
    channel_url: Option<String>,
    checked_at: Option<u64>,
    #[serde(default)]
    configured_at: u64,
    #[serde(default)]
    last_attempt: u64,
    error: Option<String>,
    manifest: Option<Manifest>,
}

impl Default for Feed {
    fn default() -> Self {
        Self {
            channel_url: Some(DEFAULT_CHANNEL.to_owned()),
            checked_at: None,
            configured_at: 0,
            last_attempt: 0,
            error: None,
            manifest: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct CachedFile {
    size: u64,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
struct CachedPack {
    pack: DocPack,
    files: BTreeMap<String, CachedFile>,
    asset_sha256: Option<String>,
}

struct Stage(PathBuf);

impl Stage {
    fn new(parent: &Path) -> Result<Self> {
        let dir = parent.join(unique_name(".staging"));
        fs::create_dir(&dir)?;
        Ok(Self(dir))
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn clock() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

fn unique_name(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}-{}",
        std::process::id(),
        clock().as_nanos(),
        STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_LOCAL_METADATA + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_LOCAL_METADATA {
        bail!("Documentation metadata exceeds the size limit");
    }
    serde_json::from_slice(&bytes).with_context(|| format!("Read {}", path.display()))
}

fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path
        .parent()
        .context("Documentation metadata has no parent directory")?;
    let temporary = parent.join(unique_name(".doc-metadata"));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        serde_json::to_writer(&mut file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn feed_path(ctx: &Context) -> PathBuf {
    ctx.home.join("doc-updates.json")
}

fn feed(ctx: &Context) -> Result<Feed> {
    let path = feed_path(ctx);
    if !path.try_exists()? {
        return Ok(Feed::default());
    }
    read_json(&path)
}

fn valid_id(id: &str) -> bool {
    id.len() <= 64
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
}

fn valid_version(version: &str) -> bool {
    version.len() <= 96
        && version
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".+-_".contains(&b))
}

fn https_url(raw: &str) -> Result<()> {
    let url = Iri::parse(raw).map_err(|_| anyhow::anyhow!("Invalid documentation release URL"))?;
    if raw.len() > 2048
        || url.scheme().as_str() != "https"
        || url.query().is_some()
        || url.fragment().is_some()
        || url
            .authority()
            .is_none_or(|auth| auth.host().is_empty() || auth.userinfo().is_some())
    {
        bail!("Use an HTTPS release URL without credentials, query, or fragment");
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_pack(pack: &DocPack) -> Result<()> {
    if !valid_id(&pack.id)
        || !valid_version(&pack.version)
        || !valid_id(&pack.stack)
        || pack.name.trim().is_empty()
        || pack.name.len() > 160
        || pack.description.len() > 4096
        || pack.license.trim().is_empty()
        || pack.license.len() > 2048
        || pack.path != format!("docs/{}/{}", pack.id, pack.version)
        || !(1..=1024).contains(&pack.shards.unwrap_or(1))
    {
        bail!("Invalid documentation pack identity or metadata");
    }
    https_url(&pack.source_url)?;
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.schema_version != 1 || manifest.releases.len() > 1000 {
        bail!("Unsupported documentation release manifest");
    }
    let mut unique = BTreeSet::new();
    for release in &manifest.releases {
        valid_pack(&release.pack)?;
        https_url(&release.url)?;
        if release.url.contains("/releases/latest/")
            || !release.url.ends_with(".tar.gz")
            || !(1..=MAX_DOWNLOAD).contains(&release.size)
            || !valid_digest(&release.sha256)
            || release.release_notes.trim().is_empty()
            || release.release_notes.len() > 16_384
            || !unique.insert((&release.pack.id, &release.pack.version))
        {
            bail!(
                "Invalid or duplicate documentation release {}@{}",
                release.pack.id,
                release.pack.version
            );
        }
    }
    Ok(())
}

fn directory(path: &Path) -> Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if error.kind() == io::ErrorKind::AlreadyExists
                && fs::symlink_metadata(path)?.file_type().is_dir() =>
        {
            Ok(())
        }
        Err(error) => Err(error).with_context(|| format!("Open directory {}", path.display())),
    }
}

fn download_dir(ctx: &Context) -> Result<PathBuf> {
    let dir = ctx.home.join("doc-downloads");
    directory(&dir)?;
    Ok(dir)
}

fn curl(protocols: &str) -> Command {
    let mut command = Command::new("/usr/bin/curl");
    command.args([
        "--disable",
        "--fail",
        "--location",
        "--max-redirs",
        "5",
        "--silent",
        "--show-error",
        "--globoff",
        "--proto",
        protocols,
        "--proto-redir",
        protocols,
        "--connect-timeout",
        "12",
        "--user-agent",
        concat!("Devline/", env!("CARGO_PKG_VERSION")),
    ]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command
}

fn fetch_manifest(ctx: &Context, url: &str) -> Result<Manifest> {
    https_url(url)?;
    let stage = Stage::new(&download_dir(ctx)?)?;
    let path = stage.0.join("manifest.json");
    let output = curl("=https")
        .args([
            "--max-time",
            "30",
            "--max-filesize",
            &MAX_MANIFEST.to_string(),
            "--output",
        ])
        .arg(&path)
        .arg("--url")
        .arg(url)
        .output()
        .context("Start documentation update check")?;
    if !output.status.success() {
        bail!(
            "Documentation channel could not be reached (curl {})",
            output.status.code().unwrap_or(-1)
        );
    }
    if fs::metadata(&path)?.len() > MAX_MANIFEST {
        bail!("Documentation manifest exceeds the size limit");
    }
    let manifest = read_json(&path)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub(crate) fn configure(ctx: &Context, url: &str) -> Result<Value> {
    let url = url.trim();
    if !url.is_empty() {
        https_url(url)?;
    }
    let channel_url = (!url.is_empty()).then(|| url.to_owned());
    {
        let _lock = lock_library(ctx)?;
        let saved = feed(ctx)?;
        if saved.channel_url != channel_url {
            atomic_json(
                &feed_path(ctx),
                &Feed {
                    channel_url,
                    checked_at: None,
                    configured_at: clock().as_nanos() as u64,
                    last_attempt: 0,
                    error: None,
                    manifest: None,
                },
            )?;
        }
    }
    status(ctx)
}

pub(crate) fn check(ctx: &Context) -> Result<Value> {
    let original = feed(ctx)?;
    let Some(channel) = original.channel_url else {
        return status(ctx);
    };
    let started = clock().as_nanos() as u64;
    let observed = fetch_manifest(ctx, &channel);
    {
        let _lock = lock_library(ctx)?;
        let mut latest = feed(ctx)?;
        if latest.channel_url.as_deref() == Some(&channel)
            && latest.configured_at == original.configured_at
            && started >= latest.last_attempt
        {
            latest.last_attempt = started;
            match observed {
                Ok(manifest) => {
                    latest.manifest = Some(manifest);
                    latest.checked_at = Some(clock().as_secs());
                    latest.error = None;
                }
                Err(error) => latest.error = Some(format!("{error:#}")),
            }
            atomic_json(&feed_path(ctx), &latest)?;
        }
    }
    status(ctx)
}

fn table_exists(db: &Connection, name: &str) -> Result<bool> {
    Ok(db.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?1 AND type='table')",
        [name],
        |row| row.get(0),
    )?)
}

pub(crate) fn preferences(db: &Connection) -> Result<BTreeMap<String, String>> {
    if !table_exists(db, "doc_preferred_releases")? {
        return Ok(BTreeMap::new());
    }
    let mut statement = db.prepare("SELECT r.pack_id,r.version FROM doc_preferred_releases r JOIN packs p ON p.id=r.pack_id AND p.version=r.version")?;
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub(crate) fn prefer_release(db: &Connection, id: &str, version: &str) -> Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS doc_preferred_releases(pack_id TEXT PRIMARY KEY,version TEXT NOT NULL);
        CREATE TRIGGER IF NOT EXISTS doc_preference_remove AFTER DELETE ON packs BEGIN
          DELETE FROM doc_preferred_releases WHERE pack_id=old.id AND version=old.version;
        END;")?;
    let changed = db.execute(
        "INSERT INTO doc_preferred_releases(pack_id,version)
        SELECT id,version FROM packs WHERE id=?1 AND version=?2
        ON CONFLICT(pack_id) DO UPDATE SET version=excluded.version",
        params![id, version],
    )?;
    if changed != 1 {
        bail!("Documentation pack {id}@{version} is not installed");
    }
    Ok(())
}

pub(crate) fn record_installed(db: &Connection, pack: &DocPack) -> Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS doc_pack_metadata(id TEXT NOT NULL,version TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(id,version));")?;
    db.execute(
        "INSERT INTO doc_pack_metadata(id,version,json) VALUES(?1,?2,?3)
        ON CONFLICT(id,version) DO UPDATE SET json=excluded.json",
        params![pack.id, pack.version, serde_json::to_string(pack)?],
    )?;
    Ok(())
}

pub(crate) fn installed_metadata(db: &Connection) -> Result<BTreeMap<(String, String), DocPack>> {
    if !table_exists(db, "doc_pack_metadata")? {
        return Ok(BTreeMap::new());
    }
    let mut statement = db.prepare("SELECT m.id,m.version,m.json FROM doc_pack_metadata m JOIN packs p ON p.id=m.id AND p.version=m.version")?;
    let mut rows = statement.query([])?;
    let mut packs = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let pack = serde_json::from_str(row.get_ref(2)?.as_str()?)?;
        packs.insert((row.get(0)?, row.get(1)?), pack);
    }
    Ok(packs)
}

fn installed_versions(db: &Connection) -> Result<BTreeMap<String, Vec<String>>> {
    let mut statement = db.prepare("SELECT id,version FROM packs")?;
    let mut rows = statement.query([])?;
    let mut versions = BTreeMap::<String, Vec<String>>::new();
    while let Some(row) = rows.next()? {
        versions.entry(row.get(0)?).or_default().push(row.get(1)?);
    }
    for releases in versions.values_mut() {
        releases.sort_by(|a, b| compare_doc_releases(b, a));
    }
    Ok(versions)
}

pub(crate) fn status(ctx: &Context) -> Result<Value> {
    let feed = feed(ctx)?;
    let db = ctx.database()?;
    let preferences = preferences(&db)?;
    let installed = installed_versions(&db)?;
    let metadata = installed_metadata(&db)?;
    let mut listed = BTreeMap::<&str, Vec<&Release>>::new();
    if let Some(manifest) = &feed.manifest {
        for release in &manifest.releases {
            listed.entry(&release.pack.id).or_default().push(release);
        }
    }
    let known = installed
        .keys()
        .map(String::as_str)
        .chain(listed.keys().copied())
        .collect::<BTreeSet<_>>();
    let mut packs = Vec::new();
    for id in known {
        let versions = installed.get(id).map_or(&[][..], Vec::as_slice);
        let active = preferences.get(id).or_else(|| versions.first());
        let mut releases = listed.remove(id).unwrap_or_default();
        releases.sort_by(|a, b| compare_doc_releases(&b.pack.version, &a.pack.version));
        let latest = releases.first().copied();
        let pack = latest
            .map(|release| &release.pack)
            .or_else(|| active.and_then(|version| metadata.get(&(id.to_owned(), version.clone()))))
            .or_else(|| ctx.catalog.docs.iter().find(|pack| pack.id == id));
        let state = match (active, latest) {
            (_, None) => "unavailable",
            (None, Some(_)) => "installable",
            (Some(version), Some(latest))
                if compare_doc_releases(&latest.pack.version, version).is_gt() =>
            {
                "available"
            }
            (Some(_), Some(_)) => "current",
        };
        packs.push(json!({"id": id, "name": pack.map_or(id, |pack| &pack.name),
            "stack": pack.map_or(id, |pack| &pack.stack), "active_version": active,
            "installed_versions": versions, "latest": latest, "releases": releases, "state": state}));
    }
    Ok(
        json!({"channel_url": feed.channel_url, "checked_at": feed.checked_at, "error": feed.error, "packs": packs}),
    )
}

fn reference<'a>(id: &'a str, version: Option<&'a str>) -> Result<(&'a str, Option<&'a str>)> {
    let (id, pinned) = id
        .split_once('@')
        .map_or((id, None), |(id, version)| (id, Some(version)));
    if !valid_id(id)
        || pinned.zip(version).is_some_and(|(a, b)| a != b)
        || pinned
            .or(version)
            .is_some_and(|version| !valid_version(version))
    {
        bail!("Use a valid documentation pack or pack@version reference");
    }
    Ok((id, pinned.or(version)))
}

pub(crate) fn rollback(ctx: &Context, request: &Value) -> Result<Value> {
    let id = crate::text_arg(request, "id")?;
    let version = request["version"].as_str();
    let (id, version) = reference(id, version)?;
    let _lock = lock_library(ctx)?;
    let mut db = ctx.database()?;
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let versions = installed_versions(&tx)?;
    let versions = versions
        .get(id)
        .context("This documentation pack is not installed")?;
    let preferred = preferences(&tx)?;
    let active = preferred
        .get(id)
        .or_else(|| versions.first())
        .context("This documentation pack is not installed")?;
    let selected = version
        .or_else(|| {
            versions
                .iter()
                .find(|version| compare_doc_releases(version, active).is_lt())
                .map(String::as_str)
        })
        .context("No earlier documentation revision is installed")?;
    prefer_release(&tx, id, selected)?;
    tx.commit()?;
    Ok(json!({"id": id, "version": selected, "active_version": selected}))
}

fn pack_dir(ctx: &Context, pack: &DocPack) -> Result<PathBuf> {
    valid_pack(pack)?;
    let root = ctx.home.join("doc-packs");
    directory(&root)?;
    let parent = root.join(&pack.id);
    directory(&parent)?;
    Ok(parent.join(&pack.version))
}

fn pack_lock_file(ctx: &Context, pack: &DocPack) -> Result<File> {
    let lock = download_dir(ctx)?.join(format!("{}@{}.lock", pack.id, pack.version));
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW)
        .open(lock)?)
}

fn safe_relative(path: &Path) -> Result<String> {
    let value = path
        .to_str()
        .context("A documentation archive contains a non-UTF-8 path")?;
    if value.is_empty()
        || value.len() > 1024
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part == RECORD)
        || value.contains(['\\', ':'])
        || value.chars().any(char::is_control)
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("Unsafe path in documentation pack");
    }
    Ok(value.to_owned())
}

fn hash_file_limit(path: &Path, limit: u64) -> Result<CachedFile> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > limit {
        bail!("Invalid documentation pack file");
    }
    let mut hash = Sha256::new();
    let size = io::copy(&mut (&mut file).take(limit + 1), &mut hash)?;
    if size != meta.len() {
        bail!("Documentation file changed while reading");
    }
    Ok(CachedFile {
        size,
        sha256: format!("{:x}", hash.finalize()),
    })
}

fn hash_file(path: &Path) -> Result<CachedFile> {
    hash_file_limit(path, MAX_FILE)
}

fn visit_tree(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, CachedFile>,
    total: &mut u64,
    copy_to: Option<&Path>,
) -> Result<()> {
    if !fs::symlink_metadata(current)?.file_type().is_dir() {
        bail!("Invalid documentation pack directory");
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if current == root && entry.file_name() == RECORD {
            continue;
        }
        let relative = safe_relative(path.strip_prefix(root)?)?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if let Some(dir) = copy_to {
                fs::create_dir_all(dir.join(&relative))?;
            }
            visit_tree(root, &path, files, total, copy_to)?;
        } else if kind.is_file() {
            if files.len() >= MAX_FILES {
                bail!("Documentation pack contains too many files");
            }
            let limit = MAX_FILE.min(MAX_UNPACKED - *total);
            let file = if let Some(dir) = copy_to {
                let output = dir.join(&relative);
                let mut source = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(&path)?;
                let mut destination = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output)?;
                let copied = io::copy(&mut (&mut source).take(limit + 1), &mut destination)?;
                if copied > limit {
                    bail!("Documentation pack exceeds unpacked limits");
                }
                hash_file_limit(&output, limit)?
            } else {
                hash_file_limit(&path, limit)?
            };
            *total += file.size;
            files.insert(relative, file);
        } else {
            bail!("Documentation pack contains a link or special file");
        }
    }
    Ok(())
}

fn validate_files(pack: &DocPack, files: &BTreeMap<String, CachedFile>) -> Result<()> {
    if files.len() > MAX_FILES
        || files
            .values()
            .try_fold(0u64, |n, file| n.checked_add(file.size))
            .is_none_or(|n| n > MAX_UNPACKED)
    {
        bail!("Documentation pack exceeds unpacked limits");
    }
    if !files.contains_key("documents.json") {
        bail!("Documentation pack is missing documents.json");
    }
    for shard in 2..=pack.shards.unwrap_or(1) {
        if !files.contains_key(&format!("documents-{shard}.json")) {
            bail!("Documentation pack is missing shard {shard}");
        }
    }
    for (path, file) in files {
        safe_relative(Path::new(path))?;
        if file.size > MAX_FILE || !valid_digest(&file.sha256) {
            bail!("Invalid cached documentation checksum metadata");
        }
    }
    Ok(())
}

fn read_record(path: &Path, id: &str, version: &str) -> Result<CachedPack> {
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        bail!("Invalid cached documentation directory");
    }
    let marker = path.join(RECORD);
    if !fs::symlink_metadata(&marker)?.file_type().is_file() {
        bail!("Invalid cached documentation metadata");
    }
    let value: CachedPack = read_json(&marker)?;
    valid_pack(&value.pack)?;
    if value.pack.id != id || value.pack.version != version {
        bail!("Cached documentation identity does not match its path");
    }
    if value
        .asset_sha256
        .as_deref()
        .is_some_and(|sha| !valid_digest(sha))
    {
        bail!("Invalid cached documentation archive checksum");
    }
    validate_files(&value.pack, &value.files)?;
    Ok(value)
}

fn verify_record(path: &Path, record: &CachedPack) -> Result<()> {
    let mut files = BTreeMap::new();
    let mut total = 0;
    visit_tree(path, path, &mut files, &mut total, None)?;
    if files != record.files {
        bail!("Cached documentation source files have changed");
    }
    Ok(())
}

fn entries(path: &Path) -> Result<Vec<fs::DirEntry>> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_dir() => {
            Ok(fs::read_dir(path)?.collect::<io::Result<_>>()?)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        _ => bail!("Invalid documentation cache path"),
    }
}

pub(crate) fn cached_packs(ctx: &Context) -> Result<Vec<(DocPack, PathBuf)>> {
    let mut packs = Vec::new();
    for pack in entries(&ctx.home.join("doc-packs"))? {
        let id = pack.file_name();
        let Some(id) = id.to_str().filter(|id| valid_id(id)) else {
            continue;
        };
        if !pack.file_type()?.is_dir() {
            continue;
        }
        for release in entries(&pack.path())? {
            let version = release.file_name();
            let Some(version) = version.to_str().filter(|version| valid_version(version)) else {
                continue;
            };
            let path = release.path();
            if let Ok(record) = read_record(&path, id, version) {
                packs.push((record.pack, path));
            }
        }
    }
    Ok(packs)
}

pub(crate) fn cached_pack(
    ctx: &Context,
    id: &str,
    version: Option<&str>,
) -> Result<Option<(DocPack, PathBuf)>> {
    let (id, version) = reference(id, version)?;
    let selected = if let Some(version) = version {
        let root = ctx.home.join("doc-packs");
        let parent = root.join(id);
        if !parent.try_exists()? {
            return Ok(None);
        }
        if !fs::symlink_metadata(&root)?.file_type().is_dir()
            || !fs::symlink_metadata(&parent)?.file_type().is_dir()
        {
            bail!("Invalid documentation cache path");
        }
        let path = parent.join(version);
        if !path.try_exists()? {
            return Ok(None);
        }
        Some((read_record(&path, id, version)?.pack, path))
    } else {
        cached_packs(ctx)?
            .into_iter()
            .filter(|(pack, _)| pack.id == id)
            .max_by(|(a, _), (b, _)| compare_doc_releases(&a.version, &b.version))
    };
    if let Some((pack, path)) = &selected {
        verify_record(path, &read_record(path, &pack.id, &pack.version)?)?;
    }
    Ok(selected)
}

fn publish_source(stage: Stage, destination: &Path, record: &CachedPack) -> Result<()> {
    validate_files(&record.pack, &record.files)?;
    if destination.try_exists()? {
        let existing = read_record(destination, &record.pack.id, &record.pack.version)?;
        verify_record(destination, &existing)?;
        if existing.files != record.files {
            bail!("This documentation revision already has different cached source files");
        }
        if record.asset_sha256.is_some() {
            atomic_json(&destination.join(RECORD), record)?;
        }
    } else {
        atomic_json(&stage.0.join(RECORD), record)?;
        fs::rename(&stage.0, destination)?;
    }
    Ok(())
}

pub(crate) fn store_original(ctx: &Context, pack: &DocPack, source: &Path) -> Result<PathBuf> {
    let destination = pack_dir(ctx, pack)?;
    let lock = pack_lock_file(ctx, pack)?;
    FileExt::lock_exclusive(&lock)?;
    if destination.try_exists()? && source.canonicalize()? == destination.canonicalize()? {
        verify_record(
            &destination,
            &read_record(&destination, &pack.id, &pack.version)?,
        )?;
        return Ok(destination);
    }
    let stage = Stage::new(destination.parent().unwrap())?;
    let mut files = BTreeMap::new();
    let mut total = 0;
    visit_tree(source, source, &mut files, &mut total, Some(&stage.0))?;
    publish_source(
        stage,
        &destination,
        &CachedPack {
            pack: pack.clone(),
            files,
            asset_sha256: None,
        },
    )?;
    Ok(destination)
}

fn extract_archive(archive: &Path, destination: &Path) -> Result<BTreeMap<String, CachedFile>> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(archive)?;
    let mut stream = GzDecoder::new(file).take(MAX_UNPACKED + (MAX_FILES as u64 * 2048) + 4096);
    let mut tar = tar::Archive::new(&mut stream);
    let mut files = BTreeMap::new();
    let mut total = 0u64;
    for entry in tar.entries()? {
        let mut entry = entry?;
        let relative = safe_relative(&entry.path()?)?;
        if !entry.header().entry_type().is_file()
            || entry.size() > MAX_FILE
            || files.len() >= MAX_FILES
        {
            bail!("Documentation archive contains a link, special file, or oversized file");
        }
        total = total
            .checked_add(entry.size())
            .filter(|n| *n <= MAX_UNPACKED)
            .context("Documentation archive exceeds unpacked limits")?;
        let output = destination.join(&relative);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .context("Duplicate or invalid archive path")?;
        let bytes = io::copy(&mut entry, &mut file)?;
        if bytes != entry.size() {
            bail!("Incomplete documentation archive entry");
        }
        files.insert(relative, hash_file(&output)?);
    }
    io::copy(&mut stream, &mut io::sink())?;
    if stream.limit() == 0 {
        bail!("Documentation archive exceeds unpacked limits");
    }
    Ok(files)
}

fn partial_size(path: &Path) -> Result<u64> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() => Ok(meta.len()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
        _ => bail!("Invalid documentation download file"),
    }
}

fn download(
    url: &str,
    path: &Path,
    expected: u64,
    protocols: &str,
    progress: &mut impl FnMut(&str, u64, u64) -> Result<()>,
) -> Result<()> {
    let mut offset = partial_size(path)?;
    if offset > expected {
        fs::remove_file(path)?;
        offset = 0;
    }
    while offset < expected {
        progress("downloading", offset, expected)?;
        let mut command = curl(protocols);
        command.stderr(Stdio::null()).args([
            "--max-time",
            "600",
            "--max-filesize",
            &expected.to_string(),
        ]);
        if offset > 0 {
            command.args(["--continue-at", "-"]);
        }
        let mut child = command
            .arg("--output")
            .arg(path)
            .arg("--url")
            .arg(url)
            .spawn()
            .context("Start documentation download")?;
        let outcome = (|| -> Result<std::process::ExitStatus> {
            loop {
                if let Some(exit) = child.try_wait()? {
                    return Ok(exit);
                }
                let bytes = partial_size(path)?;
                if bytes > expected {
                    bail!("Documentation download exceeded its announced size");
                }
                progress("downloading", bytes, expected)?;
                std::thread::sleep(Duration::from_millis(100));
            }
        })();
        let exit = match outcome {
            Ok(exit) => exit,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                if partial_size(path).unwrap_or(0) > expected {
                    let _ = fs::remove_file(path);
                }
                return Err(error);
            }
        };
        let size = partial_size(path)?;
        if size > expected {
            fs::remove_file(path)?;
            bail!("Documentation download exceeded its announced size");
        }
        if exit.success() && size == expected {
            break;
        }
        if offset > 0 && matches!(exit.code(), Some(33 | 36)) {
            // Servers without Range support can still deliver the whole archive.
            if path.try_exists()? {
                fs::remove_file(path)?;
            }
            offset = 0;
            continue;
        }
        bail!("Documentation download stopped at {size} of {expected} bytes (curl {}). Retry to resume.", exit.code().unwrap_or(-1));
    }
    Ok(())
}

pub(crate) fn install(ctx: &Context, request: &Value) -> Result<Value> {
    install_with_progress(
        ctx,
        crate::text_arg(request, "id")?,
        request["version"].as_str(),
        |_, _, _| Ok(()),
    )
}

pub(crate) fn install_with_progress(
    ctx: &Context,
    id: &str,
    version: Option<&str>,
    mut progress: impl FnMut(&str, u64, u64) -> Result<()>,
) -> Result<Value> {
    let (id, version) = reference(id, version)?;
    let state = feed(ctx)?;
    let manifest = state
        .manifest
        .context("Check for documentation releases before downloading a pack")?;
    validate_manifest(&manifest)?;
    let release = manifest
        .releases
        .iter()
        .filter(|release| {
            release.pack.id == id && version.is_none_or(|version| release.pack.version == version)
        })
        .max_by(|a, b| compare_doc_releases(&a.pack.version, &b.pack.version))
        .context("This documentation release is not in the checked channel")?;
    let destination = pack_dir(ctx, &release.pack)?;
    {
        let lock = pack_lock_file(ctx, &release.pack)?;
        loop {
            match FileExt::try_lock_exclusive(&lock) {
                Ok(()) => break,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    progress("waiting for another download", 0, release.size)?;
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => return Err(error.into()),
            }
        }
        let cached = if destination.try_exists()? {
            Some(read_record(&destination, id, &release.pack.version)?)
        } else {
            None
        };
        if let Some(cached) =
            cached.filter(|cached| cached.asset_sha256.as_deref() == Some(&release.sha256))
        {
            progress("verifying", release.size, release.size)?;
            verify_record(&destination, &cached)?;
        } else {
            let partial = download_dir(ctx)?.join(format!(
                "{}@{}-{}.part",
                id, release.pack.version, release.sha256
            ));
            download(
                &release.url,
                &partial,
                release.size,
                "=https",
                &mut progress,
            )?;
            progress("verifying", release.size, release.size)?;
            let archive = hash_file_limit(&partial, MAX_DOWNLOAD)?;
            if archive.size != release.size || archive.sha256 != release.sha256 {
                fs::remove_file(&partial)?;
                bail!("Documentation release checksum did not match the channel");
            }
            let stage = Stage::new(destination.parent().unwrap())?;
            let files = extract_archive(&partial, &stage.0)?;
            if !files.contains_key("SOURCES.md") {
                bail!("Documentation archive is missing source attribution");
            }
            let record = CachedPack {
                pack: release.pack.clone(),
                files,
                asset_sha256: Some(release.sha256.clone()),
            };
            publish_source(stage, &destination, &record)?;
            fs::remove_file(&partial)?;
        }
    }
    progress("installing", release.size, release.size)?;
    let mut result = install_doc_pack(ctx, &release.pack, &destination)?;
    {
        let _lock = lock_library(ctx)?;
        let mut db = ctx.database()?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        prefer_release(&tx, id, &release.pack.version)?;
        tx.commit()?;
    }
    result["active_version"] = json!(release.pack.version);
    result["bytes"] = json!(release.size);
    result["sha256"] = json!(release.sha256);
    Ok(result)
}

#[cfg(test)]
#[path = "doc_updates_tests.rs"]
mod tests;
