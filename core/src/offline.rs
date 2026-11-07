//! Portable bundles are a length-framed JSON manifest followed by compressed file bodies.
//! The reader never extracts arbitrary archive paths or executes imported scripts.
use crate::{
    doc_updates, install_doc_packs, lock_library, tasks::Job, templates, text_arg, Context, DocPack,
};
use anyhow::{bail, Context as _, Result};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Component, Path, PathBuf},
};

const MAGIC: &[u8] = b"DEVLINE-BUNDLE\x01\n";
const MANIFEST_LIMIT: usize = 16 * 1024 * 1024;
const FILE_LIMIT: usize = 100_000;
const SIZE_LIMIT: u64 = 5 * 1024 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct Manifest {
    schema_version: u32,
    docs: Vec<DocPack>,
    starters: Vec<templates::Installed>,
    files: Vec<Artifact>,
}

#[derive(Serialize, Deserialize)]
struct Artifact {
    path: String,
    bytes: u64,
    digest: Option<String>,
    executable: bool,
}

fn references(request: &Value, key: &str) -> Result<Vec<String>> {
    let Some(value) = request.get(key) else {
        return Ok(Vec::new());
    };
    let refs = value
        .as_array()
        .with_context(|| format!("{key} must be a list of releases"))?;
    if refs.len() > 100 {
        bail!("Select at most 100 {key} releases");
    }
    let mut output = BTreeSet::new();
    for reference in refs {
        output.insert(
            reference
                .as_str()
                .context("Expected a release reference")?
                .to_owned(),
        );
    }
    Ok(output.into_iter().collect())
}

fn doc_reference(reference: &str) -> Result<(&str, Option<&str>)> {
    let (id, version) = reference
        .split_once('@')
        .map_or((reference, None), |(id, version)| (id, Some(version)));
    if !crate::valid_doc_id(id) {
        bail!("Invalid documentation release {reference}");
    }
    if let Some(version) = version {
        crate::validate_doc_identity(id, version)?;
    }
    Ok((id, version))
}

fn exact(reference: &str) -> Result<&str> {
    reference
        .split_once('@')
        .map(|(_, version)| version)
        .with_context(|| format!("Choose an exact release for {reference}"))
}

fn require_portable_doc(id: &str, local_only: bool) -> Result<()> {
    if local_only || id == crate::apple_docs::PACK_ID {
        bail!("Documentation {id} is local-only. Choose portable docsets for this bundle.");
    }
    Ok(())
}

fn pack_source(ctx: &Context, id: &str, version: &str) -> Result<Option<(DocPack, PathBuf)>> {
    if let Some(pack) = ctx
        .catalog
        .docs
        .iter()
        .find(|pack| pack.id == id && pack.version == version)
    {
        let path = crate::safe_catalog_path(ctx, &pack.path)?;
        return Ok(Some((pack.clone(), path)));
    }
    doc_updates::cached_pack(ctx, id, Some(version))
}

fn doc_directory(pack: &DocPack) -> String {
    format!("docs/{}/{}", pack.id, pack.version)
}
fn starter_directory(release: &templates::Installed) -> String {
    format!(
        "templates/{}/{}",
        release.template.id, release.template.version
    )
}

fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && !path.contains('\\')
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn digest_ok(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn checkpoint(job: &Option<&mut Job>) -> Result<()> {
    if let Some(job) = job.as_deref() {
        job.checkpoint()?;
    }
    Ok(())
}

fn stage(job: &mut Option<&mut Job>, label: &str) -> Result<()> {
    if let Some(job) = job.as_deref_mut() {
        job.stage(label)?;
    }
    Ok(())
}

fn stream(
    input: &mut impl Read,
    output: &mut impl Write,
    length: u64,
    job: &mut Option<&mut Job>,
    label: &str,
    completed: &mut u64,
    total: u64,
) -> Result<String> {
    let mut remaining = length;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    while remaining > 0 {
        let size = remaining.min(buffer.len() as u64) as usize;
        input.read_exact(&mut buffer[..size])?;
        hash.update(&buffer[..size]);
        output.write_all(&buffer[..size])?;
        remaining -= size as u64;
        *completed += size as u64;
        if let Some(job) = job.as_deref_mut() {
            if total == 0 {
                job.checkpoint()?;
            } else {
                job.progress(label, *completed, total)?;
            }
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn inventory(
    root: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, (Artifact, PathBuf)>,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    checkpoint(job)?;
    if !safe_path(prefix) {
        bail!("Bundle contains an unsupported filename");
    }
    if files.len() >= FILE_LIMIT {
        bail!("Bundle contains too many files");
    }
    let info = fs::symlink_metadata(root)?;
    let item = if info.is_dir() {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .context("Bundle filenames must be valid UTF-8")?;
            if matches!(name, "installed.json" | ".devline-pack.json" | ".DS_Store") {
                continue;
            }
            inventory(&entry.path(), &format!("{prefix}/{name}"), files, job)?;
        }
        Artifact {
            path: prefix.into(),
            bytes: 0,
            digest: None,
            executable: false,
        }
    } else if info.is_file() {
        let mut source = File::open(root)?;
        let digest = stream(
            &mut source,
            &mut std::io::sink(),
            info.len(),
            job,
            "Hashing bundle files",
            &mut 0,
            0,
        )?;
        #[cfg(unix)]
        let executable = {
            use std::os::unix::fs::PermissionsExt;
            info.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let executable = false;
        Artifact {
            path: prefix.into(),
            bytes: info.len(),
            digest: Some(digest),
            executable,
        }
    } else {
        bail!("Bundle contains a link or special file: {prefix}");
    };
    files.insert(prefix.to_owned(), (item, root.to_path_buf()));
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<u64> {
    if manifest.schema_version != 1
        || manifest.files.is_empty()
        || manifest.files.len() > FILE_LIMIT
        || manifest.docs.len() > 100
        || manifest.starters.len() > 100
    {
        bail!("Unsupported or empty bundle manifest");
    }
    let mut roots = BTreeSet::new();
    for pack in &manifest.docs {
        require_portable_doc(&pack.id, pack.local_only)?;
        doc_reference(&format!("{}@{}", pack.id, pack.version))?;
        if pack.path != doc_directory(pack)
            || !(1..=1024).contains(&pack.shards.unwrap_or(1))
            || !roots.insert(pack.path.clone())
        {
            bail!("Invalid or duplicate documentation release");
        }
    }
    for starter in &manifest.starters {
        templates::validate_portable(&starter.template)?;
        if !digest_ok(&starter.digest)
            || starter.template.path != starter_directory(starter)
            || !roots.insert(starter.template.path.clone())
        {
            bail!("Invalid or duplicate starter release");
        }
    }
    let mut names = BTreeSet::new();
    let mut total = 0_u64;
    for artifact in &manifest.files {
        if !safe_path(&artifact.path)
            || !names.insert(artifact.path.to_lowercase())
            || !roots.iter().any(|root| {
                artifact.path == *root
                    || artifact
                        .path
                        .strip_prefix(root)
                        .is_some_and(|suffix| suffix.starts_with('/'))
            })
            || artifact
                .digest
                .as_deref()
                .is_some_and(|hash| !digest_ok(hash))
            || (artifact.digest.is_none() && (artifact.bytes != 0 || artifact.executable))
            || artifact
                .path
                .split('/')
                .any(|part| matches!(part, "installed.json" | ".devline-pack.json"))
        {
            bail!(
                "Invalid, duplicate, or unowned bundle entry {}",
                artifact.path
            );
        }
        total = total
            .checked_add(artifact.bytes)
            .context("Bundle exceeds its size limit")?;
        if total > SIZE_LIMIT {
            bail!("Bundle exceeds its size limit");
        }
    }
    for root in &roots {
        if !manifest
            .files
            .iter()
            .any(|item| item.path == *root && item.digest.is_none())
        {
            bail!("Bundle is missing release directory {root}");
        }
    }
    Ok(total)
}

pub fn export(ctx: &Context, request: &Value) -> Result<Value> {
    export_job(ctx, request, None)
}

pub(crate) fn export_job(
    ctx: &Context,
    request: &Value,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    let selected = references(request, "docs")?;
    let releases = selected
        .iter()
        .map(|reference| {
            let (id, _) = doc_reference(reference)?;
            require_portable_doc(id, false)?;
            Ok((reference, id, exact(reference)?))
        })
        .collect::<Result<Vec<_>>>()?;
    stage(&mut job, "Selecting and hashing pinned releases")?;
    let mut docs = Vec::new();
    let mut starters = Vec::new();
    let mut files = BTreeMap::new();
    for (reference, id, version) in releases {
        let (mut pack, root) = pack_source(ctx, id, version)?
            .with_context(|| format!("Original files for {reference} are unavailable. Select a bundled or cached release."))?;
        require_portable_doc(&pack.id, pack.local_only)?;
        pack.path = doc_directory(&pack);
        inventory(&root, &pack.path, &mut files, &mut job)?;
        docs.push(pack);
    }
    for reference in references(request, "starters")? {
        exact(&reference)?;
        let (mut release, root) = templates::bundle_source(ctx, &reference)?;
        release.template.path = starter_directory(&release);
        inventory(&root, &release.template.path, &mut files, &mut job)?;
        starters.push(release);
    }
    let (entries, sources): (Vec<_>, Vec<_>) = files.into_values().unzip();
    let manifest = Manifest {
        schema_version: 1,
        docs,
        starters,
        files: entries,
    };
    let total = validate_manifest(&manifest)?;
    let encoded = serde_json::to_vec(&manifest)?;
    if encoded.len() > MANIFEST_LIMIT {
        bail!("Bundle manifest is too large");
    }
    let requested = Path::new(text_arg(request, "path")?);
    let name = requested.file_name().context("Choose a bundle filename")?;
    let parent = requested
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()?;
    if parent.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a destination outside the Devline library");
    }
    let destination = parent.join(name);
    if fs::symlink_metadata(&destination).is_ok() {
        bail!("The bundle destination already exists");
    }
    let scratch = templates::Stage::new(&parent)?;
    let staged = scratch.0.join("bundle");
    let file = File::create(&staged)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(MAGIC)?;
    writer.write_all(&(encoded.len() as u64).to_le_bytes())?;
    writer.write_all(&encoded)?;
    let mut payload = GzEncoder::new(writer, Compression::fast());
    stage(&mut job, "Writing portable bundle")?;
    let mut completed = 0;
    for (artifact, source) in manifest.files.iter().zip(sources) {
        checkpoint(&job)?;
        if let Some(expected) = &artifact.digest {
            let observed = stream(
                &mut File::open(source)?,
                &mut payload,
                artifact.bytes,
                &mut job,
                "Writing portable bundle",
                &mut completed,
                total,
            )?;
            if &observed != expected {
                bail!("Source changed while exporting {}", artifact.path);
            }
        }
    }
    let mut writer = payload.finish()?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    checkpoint(&job)?;
    templates::rename_exclusive(&staged, &destination)?;
    Ok(
        json!({"path":destination,"size_bytes":fs::metadata(&destination)?.len(),
        "docs":manifest.docs.len(),"starters":manifest.starters.len()}),
    )
}

type BundlePayload = GzDecoder<BufReader<File>>;

fn open_bundle(path: &Path) -> Result<(BundlePayload, Manifest, u64)> {
    if !fs::symlink_metadata(path)?.is_file() {
        bail!("Choose a regular Devline bundle file");
    }
    let mut reader = BufReader::new(File::open(path)?);
    let length = reader.get_ref().metadata()?.len();
    let mut magic = [0; MAGIC.len()];
    reader.read_exact(&mut magic)?;
    if magic != MAGIC {
        bail!("This is not a Devline bundle");
    }
    let mut size = [0; 8];
    reader.read_exact(&mut size)?;
    let size = u64::from_le_bytes(size);
    if size == 0 || size > MANIFEST_LIMIT as u64 {
        bail!("Invalid bundle manifest length");
    }
    let mut contents = vec![0; size as usize];
    reader.read_exact(&mut contents)?;
    let manifest: Manifest = serde_json::from_slice(&contents)?;
    let total = validate_manifest(&manifest)?;
    if length <= MAGIC.len() as u64 + 8 + size || length > SIZE_LIMIT + MANIFEST_LIMIT as u64 {
        bail!("Bundle has an invalid packed size");
    }
    Ok((GzDecoder::new(reader), manifest, total))
}

fn read_files(
    reader: &mut BundlePayload,
    manifest: &Manifest,
    total: u64,
    output: Option<&Path>,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    let mut completed = 0;
    for item in &manifest.files {
        checkpoint(job)?;
        let path = output.map(|directory| directory.join(&item.path));
        let Some(expected) = &item.digest else {
            if let Some(path) = path {
                fs::create_dir_all(path)?;
            }
            continue;
        };
        let observed = if let Some(path) = path {
            fs::create_dir_all(path.parent().context("Invalid bundle file path")?)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?;
            let observed = stream(
                reader,
                &mut file,
                item.bytes,
                job,
                "Validating bundle files",
                &mut completed,
                total,
            )?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(if item.executable {
                    0o755
                } else {
                    0o644
                }))?;
            }
            observed
        } else {
            stream(
                reader,
                &mut std::io::sink(),
                item.bytes,
                job,
                "Validating bundle files",
                &mut completed,
                total,
            )?
        };
        if &observed != expected {
            bail!("Bundle checksum mismatch for {}", item.path);
        }
    }
    if reader.read(&mut [0; 1])? != 0 || !reader.get_mut().fill_buf()?.is_empty() {
        bail!("Bundle contains trailing payload data");
    }
    Ok(())
}

fn preflight(ctx: &Context, manifest: &Manifest) -> Result<Value> {
    let mut problems = Vec::new();
    let mut warnings = Vec::new();
    let mut trust = Vec::new();
    let db = ctx.database()?;
    let mut requirements = Vec::new();
    for pack in &manifest.docs {
        let release = format!("{}@{}", pack.id, pack.version);
        let mut declared = BTreeMap::new();
        for item in &manifest.files {
            if let Some(file) = item.path.strip_prefix(&format!("{}/", pack.path)) {
                if let Some(digest) = &item.digest {
                    declared.insert(file, digest);
                }
            }
        }
        if let Some((original, local)) = pack_source(ctx, &pack.id, &pack.version)? {
            if original.shards.unwrap_or(1) != pack.shards.unwrap_or(1) {
                problems.push(json!({"id":release,"reason":"Conflicts with the original documentation shard count."}));
            }
            for (index, shard) in shard_names(&original).enumerate() {
                let digest = crate::hash_doc_shard(&local, &original, index + 1)?;
                if declared.get(shard.as_str()).map(|value| value.as_str()) != Some(digest.as_str())
                {
                    problems.push(json!({"id":release,"reason":"Conflicts with original documentation files for this exact release."}));
                    break;
                }
            }
        }
        for shard in shard_names(pack) {
            if !declared.contains_key(shard.as_str()) {
                problems.push(json!({"id":release,"reason":format!("Missing {shard}.")}));
            }
        }
    }
    for starter in &manifest.starters {
        let template = &starter.template;
        let id = format!("{}@{}", template.id, template.version);
        let script = format!("{}/project/test-offline.sh", template.path);
        if !manifest
            .files
            .iter()
            .any(|item| item.path == script && item.digest.is_some())
        {
            problems.push(json!({"id":id,"reason":"Missing the starter's offline test script."}));
        }
        if !templates::target_matches(template) {
            problems.push(json!({"id":id,"reason":templates::target_detail(template)}));
        }
        match templates::bundled_digest(ctx, template)? {
            Some(digest) if digest != starter.digest => {
                problems.push(json!({"id":id,"reason":"Conflicts with a bundled starter of the same version."}));
            }
            None => trust.push(id.clone()),
            _ => {}
        }
        let destination =
            templates::installed_base(ctx, &template.id, false)?.join(&template.version);
        if fs::symlink_metadata(&destination).is_ok()
            && templates::verify_release(&destination)
                .map(|installed| installed.digest)
                .ok()
                .as_ref()
                != Some(&starter.digest)
        {
            problems.push(json!({"id":id,"reason":"Conflicts with an installed starter of the same version."}));
        }
        let runtime = templates::toolchain_report(template);
        if runtime["available"] != true {
            warnings.push(json!({"id":id,"reason":runtime["detail"]}));
        }
        requirements.push(json!({"id":id,"toolchain":template.toolchain,"minimum_version":template.minimum_version,
            "target":template.target,"available":runtime["available"],"detail":runtime["detail"]}));
        for doc in &template.docs {
            let (pack, version) = doc_reference(doc)?;
            let included = manifest
                .docs
                .iter()
                .any(|doc| doc.id == pack && version.is_none_or(|version| doc.version == version));
            let installed = db
                .query_row(
                    "SELECT 1 FROM packs WHERE id=?1 AND (?2 IS NULL OR version=?2) LIMIT 1",
                    params![pack, version],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
            if !included && !installed {
                warnings.push(json!({"id":id,"reason":format!("Related documentation {doc} is not in this bundle or installed.")}));
            }
        }
    }
    Ok(
        json!({"compatible":problems.is_empty(),"problems":problems,"warnings":warnings,"requirements":requirements,"requires_trust":!trust.is_empty(),
        "trust_starters":trust,"docs":manifest.docs,"starters":manifest.starters.iter().map(|entry| &entry.template).collect::<Vec<_>>(),
        "size_bytes":manifest.files.iter().map(|file| file.bytes).sum::<u64>(),"file_count":manifest.files.len()}),
    )
}

pub fn inspect(ctx: &Context, request: &Value) -> Result<Value> {
    let path = Path::new(text_arg(request, "path")?);
    let (mut reader, manifest, total) = open_bundle(path)?;
    read_files(&mut reader, &manifest, total, None, &mut None)?;
    let mut result = preflight(ctx, &manifest)?;
    result["unpacked_bytes"] = result["size_bytes"].take();
    result["size_bytes"] = json!(fs::metadata(path)?.len());
    Ok(result)
}

pub fn import(ctx: &Context, request: &Value) -> Result<Value> {
    import_job(ctx, request, None)
}

fn shard_names(pack: &DocPack) -> impl Iterator<Item = String> {
    (1..=pack.shards.unwrap_or(1)).map(|shard| {
        if shard == 1 {
            "documents.json".into()
        } else {
            format!("documents-{shard}.json")
        }
    })
}

fn document_digest(
    id: &str,
    title: &str,
    body: &str,
    html: Option<&str>,
    source: &str,
    category: &str,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update([u8::from(html.is_some())]);
    for field in [id, title, body, html.unwrap_or_default(), source, category] {
        hash.update((field.len() as u64).to_le_bytes());
        hash.update(field.as_bytes());
    }
    hash.finalize().into()
}

fn inspect_docs(pack: &DocPack, path: &Path) -> Result<(String, BTreeMap<String, [u8; 32]>)> {
    let shards = pack.shards.unwrap_or(1);
    let mut hash = Sha256::new();
    let mut ids = BTreeMap::new();
    for shard in 1..=shards {
        let bytes = crate::read_doc_shard(path, pack, shard)?;
        if shards > 1 {
            hash.update((bytes.len() as u64).to_le_bytes());
        }
        hash.update(&bytes);
        let docs: Vec<crate::Document> = serde_json::from_slice(&bytes)?;
        if docs.is_empty() {
            bail!("Documentation shard is empty");
        }
        for doc in docs {
            let category = doc
                .category
                .as_deref()
                .filter(|c| crate::doc_index::valid_category(c))
                .unwrap_or_else(|| crate::doc_index::category(&pack.id, &doc.id, &doc.source_url));
            let digest = document_digest(
                &doc.id,
                &doc.title,
                &doc.body,
                doc.html.as_deref(),
                &doc.source_url,
                category,
            );
            if doc.id.is_empty() || doc.title.is_empty() || ids.insert(doc.id, digest).is_some() {
                bail!("Invalid or duplicate documentation article");
            }
        }
    }
    Ok((format!("{:x}", hash.finalize()), ids))
}

pub(crate) fn import_job(
    ctx: &Context,
    request: &Value,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    stage(&mut job, "Reading bundle manifest")?;
    let (mut reader, manifest, total) = open_bundle(Path::new(text_arg(request, "path")?))?;
    let scratch = templates::Stage::new(&ctx.home.canonicalize()?)?;
    read_files(&mut reader, &manifest, total, Some(&scratch.0), &mut job)?;
    stage(&mut job, "Checking release integrity and compatibility")?;
    let check = preflight(ctx, &manifest)?;
    if check["compatible"] != true {
        bail!("Bundle has incompatible releases: {}", check["problems"]);
    }
    if check["requires_trust"] == true && request["trust_starters"] != true {
        bail!("Confirm trust for executable starter releases before importing them");
    }
    let db = ctx.database()?;
    let mut docs = Vec::new();
    for pack in manifest.docs {
        checkpoint(&job)?;
        let path = scratch.0.join(&pack.path);
        let (digest, _) = inspect_docs(&pack, &path)?;
        let installed: Option<String> = db
            .query_row(
                "SELECT digest FROM packs WHERE id=?1 AND version=?2",
                params![pack.id, pack.version],
                |row| row.get(0),
            )
            .optional()?;
        if installed
            .as_ref()
            .is_some_and(|installed| installed != &digest)
        {
            bail!(
                "Documentation {}@{} conflicts with the installed release",
                pack.id,
                pack.version
            );
        }
        docs.push((pack, path));
    }
    for starter in &manifest.starters {
        let path = scratch.0.join(&starter.template.path);
        if templates::tree_digest(&path)? != starter.digest
            || !path.join("project/test-offline.sh").is_file()
        {
            bail!("Starter {} has an invalid payload", starter.template.id);
        }
    }
    drop(db);
    stage(&mut job, "Installing verified bundle")?;
    let _lock = lock_library(ctx)?;
    checkpoint(&job)?;
    let mut published = Vec::new();
    let result = (|| -> Result<Value> {
        for (pack, path) in &docs {
            checkpoint(&job)?;
            doc_updates::store_original(ctx, pack, path)?;
        }
        let mut starters = Vec::new();
        for starter in manifest.starters {
            checkpoint(&job)?;
            let mut template = starter.template;
            let staged = scratch.0.join(&template.path);
            let base = templates::installed_base(ctx, &template.id, true)?;
            let destination = base.join(&template.version);
            if fs::symlink_metadata(&destination).is_ok() {
                if templates::verify_release(&destination)?.digest != starter.digest {
                    bail!("Starter {} changed during import", template.id);
                }
                starters
                    .push(json!({"id":template.id,"version":template.version,"installed":true}));
                continue;
            }
            if let Some(known) = ctx
                .catalog
                .templates
                .iter()
                .find(|entry| entry.id == template.id && entry.version == template.version)
            {
                template = known.clone();
            } else {
                // A portable receipt describes the exporting machine. Verify this copy here.
                template.verified = false;
            }
            let release = templates::Stage(staged);
            let result =
                templates::publish_release(&release, template, &destination, starter.verified_at)?;
            published.push(destination);
            starters.push(result);
        }
        checkpoint(&job)?;
        let docs = install_doc_packs(ctx, &docs)?;
        Ok(json!({"docs":docs,"starters":starters,"imported":true}))
    })();
    if result.is_err() {
        for path in published {
            fs::remove_dir_all(path)?;
        }
    }
    result
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct Stamp {
    length: u64,
    modified_ns: u128,
    #[cfg(unix)]
    inode: u64,
}

fn stamp(path: &Path) -> Result<Option<Stamp>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => bail!("Invalid database file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Ok(Some(Stamp {
        length: metadata.len(),
        modified_ns: metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos(),
        #[cfg(unix)]
        inode: metadata.ino(),
    }))
}

fn database_stamp(ctx: &Context) -> Result<[Option<Stamp>; 2]> {
    let wal = stamp(&ctx.home.join("library.sqlite3-wal"))?.filter(|file| file.length > 32);
    Ok([stamp(&ctx.home.join("library.sqlite3"))?, wal])
}

#[derive(Deserialize, Serialize)]
struct DocReceipt {
    digest: String,
    count: u64,
    checked_at: u64,
    files: [Option<Stamp>; 2],
}

fn check_installed_docs(
    ctx: &Context,
    db: &rusqlite::Connection,
    id: &str,
    version: &str,
    digest: &str,
    count: u64,
) -> Result<Value> {
    crate::validate_doc_identity(id, version)?;
    let folder = templates::library(ctx, &Path::new("readiness-docs").join(id), true)?;
    let receipt_path = folder.join(format!("{version}.json"));
    let files = database_stamp(ctx)?;
    if fs::symlink_metadata(&receipt_path).is_ok_and(|metadata| metadata.is_file()) {
        if let Ok(receipt) = serde_json::from_slice::<DocReceipt>(&fs::read(&receipt_path)?) {
            if receipt.files == files
                && receipt.digest == digest
                && receipt.count == count
                && count > 0
            {
                return Ok(
                    json!({"id":id,"version":version,"ready_offline":true,"count":count,
                    "digest":digest,"checked_at":receipt.checked_at,"integrity":"verified"}),
                );
            }
        }
    }
    let blocked = |detail: &str| json!({"id":id,"version":version,"ready_offline":false,"count":count,"digest":digest,"detail":detail});
    let Some((pack, directory)) = pack_source(ctx, id, version)? else {
        return Ok(blocked("Original files for this installed revision are unavailable. Install a current or cached release to verify it."));
    };
    let (expected_digest, mut documents) = inspect_docs(&pack, &directory)?;
    if digest != expected_digest || count == 0 || count != documents.len() as u64 {
        return Ok(blocked(
            "Installed documentation does not match the original release.",
        ));
    }
    let prefix = format!("{id}@{version}/");
    let mut statement = db.prepare("SELECT id,title,body,html,source_url,category,stack FROM documents WHERE pack_id=?1 AND version=?2")?;
    let mut rows = statement.query(params![id, version])?;
    while let Some(row) = rows.next()? {
        let key = row.get_ref(0)?.as_str()?.strip_prefix(&prefix);
        let Some(key) = key else {
            return Ok(blocked("An installed document has an invalid identity."));
        };
        let actual = document_digest(
            key,
            row.get_ref(1)?.as_str()?,
            row.get_ref(2)?.as_str()?,
            row.get_ref(3)?.as_str_or_null()?,
            row.get_ref(4)?.as_str()?,
            row.get_ref(5)?.as_str()?,
        );
        if documents.remove(key) != Some(actual) || row.get_ref(6)?.as_str()? != pack.stack {
            return Ok(blocked(
                "An installed document differs from the original release.",
            ));
        }
    }
    if !documents.is_empty() {
        return Ok(blocked("Some documents from this release are missing."));
    }
    let checked_at = crate::tasks::now();
    if database_stamp(ctx)? == files {
        let receipt = DocReceipt {
            digest: digest.to_owned(),
            count,
            checked_at,
            files,
        };
        let stage = templates::Stage::new(&folder)?;
        let file = stage.0.join("receipt");
        fs::write(&file, serde_json::to_vec(&receipt)?)?;
        fs::rename(file, &receipt_path)?;
    }
    Ok(
        json!({"id":id,"version":version,"ready_offline":true,"count":count,"digest":digest,
        "checked_at":checked_at,"integrity":"verified"}),
    )
}

pub fn status(ctx: &Context, request: &Value) -> Result<Value> {
    let mut docs = references(request, "docs")?;
    let mut starters = references(request, "starters")?;
    let mut profile_docs = BTreeMap::new();
    let mut unresolved_docs = Vec::new();
    let mut project = Value::Null;
    let mut runtimes = Vec::new();
    if let Some(path) = request["path"].as_str() {
        project = crate::project::context(ctx, Path::new(path))?;
        let stacks: Vec<_> = project["stacks"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        if !stacks.is_empty() {
            let mut reports = templates::runtime_reports(ctx, &stacks)?;
            for report in reports.as_array_mut().into_iter().flatten() {
                let compatible = if let Some(version) = report["version"].as_str() {
                    crate::project::runtime_compatibility(
                        Path::new(path),
                        text_arg(report, "id")?,
                        version,
                    )?
                } else {
                    None
                };
                report["compatible"] = json!(compatible);
                report["ready_offline"] =
                    json!(report["available"] == true && compatible != Some(false));
                if compatible == Some(false) {
                    report["detail"] = json!(format!(
                        "{} differs from the detected project requirement.",
                        report["detail"].as_str().unwrap_or_default()
                    ));
                }
                runtimes.push(report.take());
            }
        }
        if let Some(docsets) = project["docsets"].as_array() {
            for docset in docsets {
                let Some(id) = docset["id"].as_str() else {
                    continue;
                };
                if let Some(version) = docset["version"].as_str() {
                    profile_docs.insert(id.to_owned(), format!("{id}@{version}"));
                } else {
                    unresolved_docs
                        .push(json!({"id":id,"ready_offline":false,"detail":docset["detail"]}));
                }
            }
        }
        if let Some(reference) = project["origin"]["reference"].as_str() {
            starters.push(reference.to_owned());
        }
    }
    docs.extend(profile_docs.values().cloned());
    let mut starter_results = Vec::new();
    starters.sort();
    starters.dedup();
    for reference in &starters {
        match templates::doctor(ctx, reference) {
            Ok(result) => {
                if let Some(related) = result["template"]["docs"].as_array() {
                    for doc in related.iter().filter_map(Value::as_str) {
                        let (id, _) = doc_reference(doc)?;
                        if !profile_docs.contains_key(id)
                            && !unresolved_docs.iter().any(|entry| entry["id"] == id)
                        {
                            docs.push(doc.to_owned());
                        }
                    }
                }
                starter_results.push(result);
            }
            Err(error) => starter_results
                .push(json!({"id":reference,"ready_offline":false,"error":format!("{error:#}")})),
        }
    }
    docs.sort();
    docs.dedup();
    let db = ctx.database()?;
    let mut doc_results = unresolved_docs;
    for reference in docs {
        let (id, version) = doc_reference(&reference)?;
        let mut stmt = db.prepare(
            "SELECT version,digest,count FROM packs WHERE id=?1 AND (?2 IS NULL OR version=?2)",
        )?;
        let installed = stmt
            .query_map(params![id, version], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u64>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .max_by(|a, b| crate::updates::compare_doc_releases(&a.0, &b.0));
        if let Some((version, digest, count)) = installed {
            match check_installed_docs(ctx, &db, id, &version, &digest, count) {
                Ok(result) => doc_results.push(result),
                Err(error) => doc_results.push(json!({"id":id,"version":version,"ready_offline":false,"detail":format!("{error:#}")})),
            }
        } else {
            doc_results.push(json!({"id":id,"version":version,"ready_offline":false,"detail":"Install this documentation release."}));
        }
    }
    let selected = !starter_results.is_empty() || !doc_results.is_empty() || !runtimes.is_empty();
    let ready = selected
        && starter_results
            .iter()
            .chain(&doc_results)
            .chain(&runtimes)
            .all(|item| item["ready_offline"] == true);
    Ok(
        json!({"ready_offline":ready,"status":if ready { "ready_offline" } else if selected { "needs_preparation" } else { "no_selection" },
        "docs":doc_results,"starters":starter_results,"runtimes":runtimes,"project":project}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Catalog, Template};
    use std::io::{Seek, SeekFrom};

    #[test]
    fn an_unsafe_legacy_version_cannot_write_a_readiness_receipt() -> Result<()> {
        let staging = templates::Stage::new(&std::env::temp_dir())?;
        let sentinel = staging.0.join("outside.json");
        fs::write(&sentinel, "untouched")?;
        let home = staging.0.join("home");
        fs::create_dir(&home)?;
        let ctx = Context {
            home,
            catalog_dir: staging.0.clone(),
            catalog: Catalog {
                schema_version: 1,
                docs: Vec::new(),
                templates: Vec::new(),
            },
        };
        let bad_version = sentinel.with_extension("");
        let db = ctx.database()?;
        assert!(
            check_installed_docs(&ctx, &db, "go", bad_version.to_str().unwrap(), "", 1)
                .unwrap_err()
                .to_string()
                .contains("identity")
        );
        assert_eq!(fs::read_to_string(sentinel)?, "untouched");
        assert!(!ctx.home.join("readiness-docs").exists());
        Ok(())
    }

    #[test]
    fn local_only_and_reserved_docs_cannot_cross_bundle_boundaries() -> Result<()> {
        let work = templates::Stage::new(&std::env::temp_dir())?;
        let home = work.0.join("home");
        fs::create_dir(&home)?;
        let pack = |id: &str, local_only: bool| {
            serde_json::from_value::<DocPack>(json!({
                "id":id,"name":"Sample","stack":"swift","version":"1.0.0",
                "description":"Sample documentation","source_url":"https://example.com",
                "license":"Example","path":format!("docs/{id}/1.0.0"),"local_only":local_only,
            }))
        };
        let reserved = pack("swiftui", false)?;
        let local = pack("device-reference", true)?;
        let local_source = work.0.join(&local.path);
        fs::create_dir_all(&local_source)?;
        fs::write(
            local_source.join("documents.json"),
            serde_json::to_vec(&json!([{
                "id":"sample","title":"Sample","body":"This reference stays on the device.",
                "source_url":"https://example.com/sample",
            }]))?,
        )?;
        let ctx = Context {
            home,
            catalog_dir: work.0.clone(),
            catalog: Catalog {
                schema_version: 1,
                docs: vec![pack("go", false)?, reserved.clone(), local.clone()],
                templates: Vec::new(),
            },
        };
        let output = work.0.join("output.devlinebundle");
        // Both original paths are absent. The reserved ID must be caught before
        // discovery of even an earlier, otherwise portable, selection.
        for docs in [
            vec!["go@1.0.0", "swiftui@1.0.0"],
            vec!["device-reference@1.0.0"],
        ] {
            let error = export(&ctx, &json!({"path":output,"docs":docs})).unwrap_err();
            assert!(error.to_string().contains("local-only"), "{error:#}");
            assert!(!output.exists());
        }
        for pack in [reserved, local.clone()] {
            let manifest = json!({
                "schema_version":1,"starters":[],
                "files":[{"path":pack.path,"bytes":0,"digest":null,"executable":false}],
                "docs":[pack],
            });
            let encoded = serde_json::to_vec(&manifest)?;
            let input = work.0.join("input.devlinebundle");
            let mut file = File::create(&input)?;
            file.write_all(MAGIC)?;
            file.write_all(&(encoded.len() as u64).to_le_bytes())?;
            file.write_all(&encoded)?;
            file.write_all(b"invalid gzip payload")?;
            drop(file);

            // Manifest policy must win before either path reads the payload.
            let request = json!({"path":input,"trust_starters":true});
            for action in [inspect, import] {
                let error = action(&ctx, &request).unwrap_err();
                assert!(error.to_string().contains("local-only"), "{error:#}");
            }
            assert!(fs::read_dir(&ctx.home)?.next().is_none());
        }
        crate::install_doc_pack(&ctx, &local, &local_source)?;
        let ready = status(&ctx, &json!({"docs":["device-reference@1.0.0"]}))?;
        assert_eq!(ready["ready_offline"], true);
        Ok(())
    }

    #[test]
    fn bundle_round_trip_checks_trust_digests_and_installed_content() -> Result<()> {
        let work = templates::Stage::new(&std::env::temp_dir())?;
        let catalog_dir = work.0.join("catalog");
        let pack = DocPack {
            id: "go".into(),
            name: "Go".into(),
            stack: "go".into(),
            version: "1.2.3".into(),
            description: "Sample documentation".into(),
            source_url: "https://go.dev/".into(),
            license: "BSD-3-Clause".into(),
            path: "docs/go/1.2.3".into(),
            markdown: None,
            shards: None,
            upstream_version: None,
            snapshot_date: None,
            coverage: Vec::new(),
            omissions: Vec::new(),
            local_only: false,
        };
        let template = Template {
            id: "go/sample".into(),
            name: "Sample".into(),
            stack: "go".into(),
            version: "1.2.3".into(),
            description: "Starter".into(),
            app_type: "cli".into(),
            path: "templates/go/sample/1.2.3".into(),
            toolchain: "go".into(),
            minimum_version: "1.0.0".into(),
            minimum_macos: None,
            test_command: "./test-offline.sh".into(),
            docs: vec!["go@1.2.3".into()],
            services: Vec::new(),
            verified: true,
            target: "any".into(),
        };
        let docs = catalog_dir.join(&pack.path);
        fs::create_dir_all(&docs)?;
        fs::write(
            docs.join("documents.json"),
            serde_json::to_vec(&json!([{
                "id":"sample","title":"Sample","body":"Original", "source_url":"https://go.dev/sample"
            }]))?,
        )?;
        let project = catalog_dir.join(&template.path).join("project");
        fs::create_dir_all(&project)?;
        fs::write(project.join("test-offline.sh"), "#!/bin/sh\nexit 0\n")?;
        let source_home = work.0.join("source");
        let target_home = work.0.join("target");
        fs::create_dir(&source_home)?;
        fs::create_dir(&target_home)?;
        let source = Context {
            home: source_home,
            catalog_dir: catalog_dir.clone(),
            catalog: Catalog {
                schema_version: 1,
                docs: vec![pack],
                templates: vec![template],
            },
        };
        let target = Context {
            home: target_home,
            catalog_dir,
            catalog: Catalog {
                schema_version: 1,
                docs: Vec::new(),
                templates: Vec::new(),
            },
        };
        let path = work.0.join("selected.devlinebundle");
        export(
            &source,
            &json!({"path":path,"docs":["go@1.2.3"],"starters":["go/sample@1.2.3"]}),
        )?;
        let preflight = inspect(&target, &json!({"path":path}))?;
        assert_eq!(preflight["requires_trust"], true);
        assert!(import(&target, &json!({"path":path})).is_err());
        assert!(templates::list(&target)?.as_array().unwrap().is_empty());
        assert_eq!(
            import(&target, &json!({"path":path,"trust_starters":true}))?["imported"],
            true
        );
        let starter = templates::list(&target)?;
        assert_eq!(starter[0]["verified"], false);
        assert_eq!(starter[0]["installed"], true);
        let request = json!({"docs":["go@1.2.3"]});
        assert_eq!(status(&target, &request)?["ready_offline"], true);
        target
            .database()?
            .execute("UPDATE documents SET body='Altered'", [])?;
        assert_eq!(status(&target, &request)?["ready_offline"], false);
        let mut archive = OpenOptions::new().read(true).write(true).open(&path)?;
        archive.seek(SeekFrom::End(-1))?;
        let mut byte = [0];
        archive.read_exact(&mut byte)?;
        archive.seek(SeekFrom::End(-1))?;
        archive.write_all(&[byte[0] ^ 1])?;
        assert!(inspect(&target, &json!({"path":path})).is_err());
        Ok(())
    }
}
