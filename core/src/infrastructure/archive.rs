//! Canonical Docker archives contain one untagged image and no repository/index metadata.
use super::*;
use flate2::{
    bufread::{GzDecoder, MultiGzDecoder},
    write::GzEncoder,
    Compression,
};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, BufWriter};

const MAGIC: &[u8] = b"DEVLINE-INFRA\x01\n";
const MAX_UNPACKED: u64 = 64 * 1024 * 1024 * 1024;
const MAX_PORTABLE: u64 = 128 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 20_000;

#[derive(Deserialize, Serialize)]
struct ImageManifest {
    #[serde(rename = "Config")]
    config: String,
    #[serde(rename = "Layers")]
    layers: Vec<String>,
    #[serde(rename = "RepoTags")]
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct Config {
    os: String,
    architecture: String,
    rootfs: RootFs,
}
#[derive(Deserialize)]
struct RootFs {
    diff_ids: Vec<String>,
}

fn path(value: &str) -> Result<&str> {
    let value = value.strip_prefix("./").unwrap_or(value);
    if value.is_empty()
        || value.len() > 512
        || !value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
        })
    {
        bail!("Unsafe Docker archive path");
    }
    Ok(value)
}

fn index_metadata(value: &str) -> bool {
    matches!(value, "index.json" | "oci-layout" | "repositories")
}

fn one_manifest(input: impl Read, canonical: bool) -> Result<ImageManifest> {
    let mut bytes = Vec::new();
    input.take(MAX_METADATA + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_METADATA {
        bail!("Docker manifest is too large");
    }
    let mut entries: Vec<ImageManifest> = serde_json::from_slice(&bytes)?;
    if entries.len() != 1 {
        bail!("Archive must contain exactly one Docker image");
    }
    // Docker's Go JSON decoder accepts field names case-insensitively. Exact
    // canonical bytes reject hidden RepoTags, Parent, duplicate and unknown keys.
    if canonical && serde_json::to_vec(&entries)? != bytes {
        bail!("Noncanonical Docker manifest metadata");
    }
    let manifest = entries.remove(0);
    path(&manifest.config)?;
    let mut layers = BTreeSet::new();
    if manifest.layers.len() > 512
        || manifest
            .layers
            .iter()
            .any(|name| path(name).is_err() || !layers.insert(name))
    {
        bail!("Invalid or duplicate Docker layer path");
    }
    if layers.contains(&manifest.config) {
        bail!("Docker config cannot also be a layer");
    }
    Ok(manifest)
}

struct HashWriter {
    file: BufWriter<File>,
    hash: Sha256,
    bytes: u64,
}
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.saturating_add(bytes.len() as u64) > MAX_IMAGE {
            return Err(io::Error::other(
                "Compressed image archive exceeds the size limit",
            ));
        }
        let written = self.file.write(bytes)?;
        self.hash.update(&bytes[..written]);
        self.bytes += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn append(
    writer: &mut tar::Builder<impl Write>,
    name: &str,
    bytes: u64,
    input: impl Read,
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_mode(0o644);
    header.set_size(bytes);
    writer.append_data(&mut header, name, input)?;
    Ok(())
}

pub(super) fn save_tar(input: impl Read, destination: &Path) -> Result<(String, u64)> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let compressed = GzEncoder::new(
        HashWriter {
            file: BufWriter::new(file),
            hash: Sha256::new(),
            bytes: 0,
        },
        Compression::fast(),
    );
    let mut output = tar::Builder::new(compressed);
    let mut original = tar::Archive::new(input);
    let mut files = BTreeSet::new();
    let mut manifest = None;
    let mut total = 0_u64;
    for entry in original.entries()? {
        let mut entry = entry?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let original_name = entry
            .path()?
            .to_str()
            .context("Non-UTF-8 Docker archive path")?
            .to_owned();
        let name = path(&original_name)?;
        if !entry.header().entry_type().is_file()
            || !files.insert(name.to_owned())
            || files.len() > MAX_ENTRIES
        {
            bail!("Invalid Docker archive entry");
        }
        total = total
            .checked_add(entry.size())
            .context("Docker archive exceeds its size limit")?;
        if total > MAX_UNPACKED {
            bail!("Docker archive exceeds its size limit");
        }
        if index_metadata(name) {
            continue;
        }
        if name == "manifest.json" {
            manifest = Some(one_manifest(&mut entry, false)?);
        } else {
            append(&mut output, name, entry.size(), entry)?;
        }
    }
    io::copy(&mut original.into_inner(), &mut io::sink())?;
    let mut manifest = manifest.context("Docker save did not produce a Docker manifest")?;
    manifest.tags = None;
    manifest.config = path(&manifest.config)?.to_owned();
    for layer in &mut manifest.layers {
        *layer = path(layer)?.to_owned();
    }
    let bytes = serde_json::to_vec(&vec![manifest])?;
    append(
        &mut output,
        "manifest.json",
        bytes.len() as u64,
        bytes.as_slice(),
    )?;
    let mut writer = output.into_inner()?.finish()?;
    writer.flush()?;
    writer.file.get_ref().sync_all()?;
    Ok((format!("{:x}", writer.hash.finalize()), writer.bytes))
}

pub(super) fn regular(path: &Path, limit: u64) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > limit {
        bail!("Invalid image archive file");
    }
    Ok(file)
}

fn stream(
    input: &mut impl Read,
    output: &mut impl Write,
    limit: u64,
    job: &mut Option<&mut Job>,
    progress: Option<(&str, &mut u64, u64)>,
) -> Result<(String, u64)> {
    let mut total = 0_u64;
    let mut progress = progress;
    let mut hash = Sha256::new();
    let mut bytes = [0_u8; 128 * 1024];
    loop {
        checkpoint(job)?;
        let size = input.read(&mut bytes)?;
        if size == 0 {
            break;
        }
        total += size as u64;
        if total > limit {
            bail!("Image archive exceeds its size limit");
        }
        hash.update(&bytes[..size]);
        output.write_all(&bytes[..size])?;
        if let Some((label, completed, expected)) = progress.as_mut() {
            **completed += size as u64;
            if let Some(job) = job.as_deref_mut() {
                job.progress(label, **completed, *expected)?;
            }
        }
    }
    Ok((format!("{:x}", hash.finalize()), total))
}

pub(super) fn verify_checksum(
    path: &Path,
    receipt: &Receipt,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    let mut file = regular(path, MAX_IMAGE).context("Read the independent image archive")?;
    if file.metadata()?.len() != receipt.size_bytes {
        bail!("Image archive length does not match its receipt");
    }
    let (hash, size) = stream(&mut file, &mut io::sink(), receipt.size_bytes, job, None)?;
    if size != receipt.size_bytes || hash != receipt.archive_sha256 {
        bail!("Image archive checksum does not match its receipt");
    }
    Ok(())
}

pub(super) fn verify_tar(file: &Path, service: &Service, job: &mut Option<&mut Job>) -> Result<()> {
    stage(job, "Verifying image config and layer identities")?;
    let input = GzDecoder::new(BufReader::new(regular(file, MAX_IMAGE)?));
    let mut archive = tar::Archive::new(input);
    let mut manifest = None;
    let mut configs = BTreeMap::new();
    let mut files = BTreeMap::new();
    let mut total = 0_u64;
    for entry in archive.entries()?.raw(true) {
        checkpoint(job)?;
        let entry = entry?;
        if !entry.header().entry_type().is_file() || files.len() >= MAX_ENTRIES {
            bail!("Invalid file in Docker archive");
        }
        let original = entry
            .path()?
            .to_str()
            .context("Non-UTF-8 Docker archive path")?
            .to_owned();
        let name = path(&original)?;
        if name != original || index_metadata(name) || files.contains_key(name) {
            bail!("Unexpected Docker archive metadata or duplicate file");
        }
        if name == "manifest.json" {
            if manifest.is_some() {
                bail!("Duplicate Docker manifest");
            }
            let record = one_manifest(entry, true)?;
            if record.tags.as_ref().is_some_and(|tags| !tags.is_empty()) {
                bail!("Image archive must not restore repository tags");
            }
            manifest = Some(record);
            continue;
        }
        let size = entry.size();
        let mut reader = BufReader::new(entry);
        let limit = MAX_UNPACKED.saturating_sub(total);
        let (hash, bytes) = if reader.fill_buf()?.starts_with(&[0x1f, 0x8b]) {
            stream(
                &mut MultiGzDecoder::new(reader),
                &mut io::sink(),
                limit,
                job,
                None,
            )?
        } else if size <= MAX_METADATA && reader.fill_buf()?.first() == Some(&b'{') {
            let mut body = Vec::new();
            let (hash, size) = stream(&mut reader, &mut body, limit, job, None)?;
            if format!("sha256:{hash}") == service.image.config_digest {
                configs.insert(name.to_owned(), serde_json::from_slice::<Config>(&body)?);
            }
            (hash, size)
        } else {
            stream(&mut reader, &mut io::sink(), limit, job, None)?
        };
        total += bytes;
        files.insert(name.to_owned(), format!("sha256:{hash}"));
    }
    let mut input = archive.into_inner();
    let mut trailing = [0_u8; 1024];
    let mut padding = 0;
    loop {
        let size = input.read(&mut trailing)?;
        if size == 0 {
            break;
        }
        padding += size;
        if padding > 1024 * 1024 || trailing[..size].iter().any(|byte| *byte != 0) {
            bail!("Unexpected trailing Docker archive data");
        }
    }
    if !input.get_mut().fill_buf()?.is_empty() {
        bail!("Unexpected trailing compressed image data");
    }
    let manifest = manifest.context("Docker archive is missing its manifest")?;
    let config = configs
        .remove(&manifest.config)
        .context("Docker archive config differs from the pinned image ID")?;
    let (os, arch) = service
        .image
        .platform
        .split_once('/')
        .context("Invalid image platform")?;
    if config.os != os
        || config.architecture != arch
        || config.rootfs.diff_ids.len() != manifest.layers.len()
    {
        bail!("Docker archive platform or layer count differs from its image config");
    }
    for (layer, expected) in manifest.layers.iter().zip(&config.rootfs.diff_ids) {
        if !digest(expected) || files.get(layer) != Some(expected) {
            bail!("Docker image layer digest mismatch");
        }
    }
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct Portable {
    schema_version: u32,
    images: Vec<Receipt>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pins: BTreeMap<String, Image>,
}

pub(super) fn export(
    ctx: &Context,
    store: &Store,
    services: &[Service],
    request: &Value,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    stage(&mut job, "Selecting archived infrastructure images")?;
    let input = request["images"]
        .as_array()
        .context("Choose exact infrastructure releases to export")?;
    if input.is_empty() || input.len() > 100 {
        bail!("Choose between one and 100 releases");
    }
    let _lock = store.lock(&mut job)?;
    let index = store.index()?;
    let mut records = BTreeMap::new();
    let mut pins = BTreeMap::new();
    for value in input {
        let reference = value.as_str().context("Expected service@version")?;
        if !reference.contains('@') {
            bail!("Choose an exact infrastructure release");
        }
        let selected = select(services, reference)?;
        let entry = index
            .get(&selected.key())
            .with_context(|| format!("Prepare {reference} before exporting"))?;
        records.insert(selected.key(), entry.receipt.clone());
        pins.insert(selected.key(), entry.service.image.clone());
    }
    let manifest = Portable {
        schema_version: 1,
        images: records.into_values().collect(),
        pins,
    };
    let total: u64 = manifest
        .images
        .iter()
        .map(|receipt| receipt.size_bytes)
        .sum();
    if total > MAX_PORTABLE {
        bail!("Portable infrastructure archive exceeds its size limit");
    }
    let requested = Path::new(text_arg(request, "path")?);
    let parent = requested
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()?;
    let name = requested.file_name().context("Choose an export filename")?;
    if parent.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a destination outside the Devline library");
    }
    let target = parent.join(name);
    let staging = templates::Stage::new(&parent)?;
    let temporary = staging.0.join("export");
    let mut file = BufWriter::new(File::create(&temporary)?);
    let metadata = serde_json::to_vec(&manifest)?;
    file.write_all(MAGIC)?;
    file.write_all(&(metadata.len() as u64).to_le_bytes())?;
    file.write_all(&metadata)?;
    let mut completed = 0;
    for receipt in &manifest.images {
        let mut source = regular(&store.blob(receipt), MAX_IMAGE)?;
        let (hash, size) = stream(
            &mut source,
            &mut file,
            receipt.size_bytes,
            &mut job,
            Some(("Writing portable image archive", &mut completed, total)),
        )?;
        if hash != receipt.archive_sha256 || size != receipt.size_bytes {
            bail!("Image archive changed during export");
        }
    }
    file.flush()?;
    file.get_ref().sync_all()?;
    checkpoint(&job)?;
    templates::rename_exclusive(&temporary, &target)
        .context("Export destination already exists or is unavailable")?;
    File::open(parent)?.sync_all()?;
    Ok(
        json!({"path":target,"images":manifest.images.len(),"size_bytes":fs::metadata(&target)?.len()}),
    )
}

fn read_portable(path: &Path) -> Result<(BufReader<File>, Portable, String)> {
    let file = regular(path, MAX_PORTABLE + MAX_METADATA + 64)?;
    let length = file.metadata()?.len();
    let mut input = BufReader::new(file);
    let mut magic = [0_u8; MAGIC.len()];
    input.read_exact(&mut magic)?;
    if magic != MAGIC {
        bail!("This is not a Devline infrastructure archive");
    }
    let mut size = [0_u8; 8];
    input.read_exact(&mut size)?;
    let size = u64::from_le_bytes(size);
    if size == 0 || size > MAX_METADATA {
        bail!("Invalid infrastructure archive manifest length");
    }
    let mut metadata = vec![0_u8; size as usize];
    input.read_exact(&mut metadata)?;
    let manifest: Portable = serde_json::from_slice(&metadata)?;
    if manifest.schema_version != 1
        || manifest.images.is_empty()
        || manifest.images.len() > 100
        || manifest.pins.len() > manifest.images.len()
    {
        bail!("Unsupported or empty infrastructure archive manifest");
    }
    let payload = manifest.images.iter().try_fold(0_u64, |sum, receipt| {
        sum.checked_add(receipt.size_bytes)
            .filter(|total| *total <= MAX_PORTABLE)
            .context("Portable image archive exceeds its size limit")
    })?;
    if length != MAGIC.len() as u64 + 8 + size + payload {
        bail!("Portable archive size differs from the manifest");
    }
    Ok((input, manifest, format!("{:x}", Sha256::digest(metadata))))
}

fn releases(manifest: Portable, services: &[Service]) -> Result<Vec<(Service, Receipt, bool)>> {
    let mut seen = BTreeSet::new();
    let mut releases = Vec::new();
    for receipt in manifest.images {
        let reference = format!("{}@{}", receipt.id, receipt.version);
        let (service, known) = match select(services, &reference) {
            Ok(service) => (service, true),
            Err(_) => {
                let key = format!("{reference}/{}", receipt.platform);
                let pin = manifest.pins.get(&key).with_context(|| {
                    format!("No trusted or transferable publisher pin for {reference}")
                })?;
                (registry::transferred_pin(services, &receipt, pin)?, false)
            }
        };
        receipt.validate(&service)?;
        if !seen.insert(service.key()) {
            bail!("Duplicate release in infrastructure archive");
        }
        releases.push((service, receipt, known));
    }
    Ok(releases)
}

pub(super) fn inspect(services: &[Service], request: &Value) -> Result<Value> {
    let (_, manifest, manifest_sha256) = read_portable(Path::new(text_arg(request, "path")?))?;
    let releases = releases(manifest, services)?;
    let requires_trust = releases.iter().any(|(_, _, known)| !*known);
    let images:Vec<_> = releases.iter().map(|(service,receipt,known)| json!({
        "id":receipt.id,"version":receipt.version,"platform":receipt.platform,
        "source":receipt.source,"image_id":receipt.image_id,"size_bytes":receipt.size_bytes,
        "name":service.name,"known":known,"detail":if *known {"Publisher pin is already trusted by this library."}
            else {"The archive supplies this publisher pin. Confirm its source before importing."}
    })).collect();
    Ok(json!({"images":images,"requires_trust":requires_trust,"manifest_sha256":manifest_sha256}))
}

pub(super) fn import(
    store: &Store,
    services: &[Service],
    request: &Value,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    stage(&mut job, "Reading portable infrastructure archive")?;
    let (mut input, manifest, manifest_sha256) =
        read_portable(Path::new(text_arg(request, "path")?))?;
    if request["manifest_sha256"]
        .as_str()
        .is_some_and(|expected| expected != manifest_sha256)
    {
        bail!("Infrastructure archive changed after preview");
    }
    let releases = releases(manifest, services)?;
    if releases.iter().any(|(_, _, known)| !*known) && request["trust_images"] != true {
        bail!("This archive contains a new publisher pin. Inspect and explicitly trust it before importing.");
    }
    let total = releases
        .iter()
        .map(|(_, receipt, _)| receipt.size_bytes)
        .sum();
    let staging = templates::Stage::new(&store.images)?;
    let mut completed = 0;
    for (index, (service, receipt, _)) in releases.iter().enumerate() {
        let path = staging.0.join(format!("{index}.tar.gz"));
        let mut output = File::create(&path)?;
        let mut content = (&mut input).take(receipt.size_bytes);
        let (hash, bytes) = stream(
            &mut content,
            &mut output,
            receipt.size_bytes,
            &mut job,
            Some(("Validating portable image archive", &mut completed, total)),
        )?;
        output.sync_all()?;
        if hash != receipt.archive_sha256 || bytes != receipt.size_bytes {
            bail!("Portable image checksum mismatch");
        }
        verify_tar(&path, service, &mut job)?;
    }
    if input.read(&mut [0_u8; 1])? != 0 {
        bail!("Infrastructure archive has trailing data");
    }
    stage(&mut job, "Publishing verified local archives")?;
    let _lock = store.lock(&mut job)?;
    let mut index = store.index()?;
    for (service, receipt, _) in &releases {
        if let Some(existing) = index.get(&service.key()) {
            existing.receipt.validate(service)?;
            if existing.receipt.archive_sha256 != receipt.archive_sha256 {
                bail!("This immutable image release already has another archive");
            }
        }
    }
    checkpoint(&job)?;
    let mut installed = Vec::new();
    for (position, (service, mut receipt, _)) in releases.into_iter().enumerate() {
        let stamp = store.publish_blob(&staging.0.join(format!("{position}.tar.gz")), &receipt)?;
        receipt.verified_at = crate::tasks::now();
        installed.push(receipt.clone());
        index.insert(
            service.key(),
            Stored {
                service,
                receipt,
                stamp,
            },
        );
    }
    store.write_index(&index)?;
    Ok(json!({"images":installed,"imported":true}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<(Service, Vec<u8>, Vec<u8>)> {
        let mut layer = tar::Builder::new(Vec::new());
        append(&mut layer, "fixture.txt", 4, b"test".as_slice())?;
        let layer = layer.into_inner()?;
        let (os, architecture) = platform().split_once('/').unwrap();
        let config = serde_json::to_vec(&json!({"os":os,"architecture":architecture,
            "rootfs":{"diff_ids":[format!("sha256:{:x}",Sha256::digest(&layer))]}}))?;
        let service = serde_json::from_value(json!({
            "id":"fixture","name":"Fixture","version":"1","category":"test","description":"Fixture",
            "image":{"registry":"docker.io","repository":"library/fixture","tag":"1",
                "digest":format!("sha256:{}","a".repeat(64)),
                "config_digest":format!("sha256:{:x}",Sha256::digest(&config)),"platform":platform()},
            "healthcheck":{"command":["CMD","true"],"interval_seconds":1,"timeout_seconds":1,"retries":1},
            "provenance":{}
        }))?;
        Ok((service, config, layer))
    }

    fn image<W: Write>(output: W, config: &[u8], layer: &[u8], manifest: &[u8]) -> Result<W> {
        let mut tar = tar::Builder::new(output);
        for (path, bytes) in [
            ("config.json", config),
            ("layer.tar", layer),
            ("manifest.json", manifest),
        ] {
            append(&mut tar, path, bytes.len() as u64, bytes)?;
        }
        Ok(tar.into_inner()?)
    }

    #[test]
    fn normalizes_tags_and_roundtrips_portable_archive_without_docker() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir())?;
        let (service, config, layer) = fixture()?;
        let original = image(Vec::new(),&config,&layer,
            br#"[{"Config":"config.json","Layers":["layer.tar"],"RepoTags":["user/existing:latest"]}]"#)?;
        let home = root.0.join("home");
        fs::create_dir(&home)?;
        let ctx = Context {
            home,
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        let store = Store::open(&ctx)?;
        let temporary = root.0.join("save.gz");
        let (hash, bytes) = save_tar(original.as_slice(), &temporary)?;
        verify_tar(&temporary, &service, &mut None)?;
        let receipt = Receipt::from(&service, hash, bytes);
        let stamp = store.publish_blob(&temporary, &receipt)?;
        store.write_index(&BTreeMap::from([(
            service.key(),
            Stored {
                service: service.clone(),
                receipt: receipt.clone(),
                stamp,
            },
        )]))?;

        let file = root.0.join("portable.devlineinfra");
        export(
            &ctx,
            &store,
            std::slice::from_ref(&service),
            &json!({"path":file,"images":["fixture@1"]}),
            None,
        )?;
        let destination = Context {
            home: root.0.join("other-home"),
            ..ctx
        };
        fs::create_dir(&destination.home)?;
        let imported = Store::open(&destination)?;
        import(
            &imported,
            std::slice::from_ref(&service),
            &json!({"path":file}),
            None,
        )?;
        assert_eq!(
            imported.index()?[&service.key()].receipt.archive_sha256,
            receipt.archive_sha256
        );
        verify_checksum(&imported.blob(&receipt), &receipt, &mut None)?;
        Ok(())
    }

    #[test]
    fn rejects_forged_archive_receipt_with_docker_casefolded_tags() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir())?;
        let (service, config, layer) = fixture()?;
        let malicious = image(GzEncoder::new(Vec::new(),Compression::fast()),&config,&layer,
            br#"[{"Config":"config.json","Layers":["layer.tar"],"RepoTags":null,"repotags":["user/existing:latest"]}]"#)?.finish()?;
        let receipt = Receipt::from(
            &service,
            format!("{:x}", Sha256::digest(&malicious)),
            malicious.len() as u64,
        );
        let metadata = serde_json::to_vec(&Portable {
            schema_version: 1,
            images: vec![receipt.clone()],
            pins: BTreeMap::new(),
        })?;
        let file = root.0.join("untrusted.devlineinfra");
        let mut out = File::create(&file)?;
        out.write_all(MAGIC)?;
        out.write_all(&(metadata.len() as u64).to_le_bytes())?;
        out.write_all(&metadata)?;
        out.write_all(&malicious)?;
        let ctx = Context {
            home: root.0.clone(),
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        let store = Store::open(&ctx)?;
        let error = import(&store, &[service], &json!({"path":file}), None).unwrap_err();
        assert!(
            error.to_string().contains("Noncanonical Docker manifest"),
            "{error:#}"
        );
        assert!(store.index()?.is_empty());
        assert!(!store.blob(&receipt).exists());
        Ok(())
    }

    #[test]
    fn unknown_compatible_pin_requires_explicit_trust_and_matching_preview() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir())?;
        let (mut baseline, config, layer) = fixture()?;
        baseline.version = "1.0.0".into();
        baseline.image.tag = "1.0.0".into();
        let mut update = baseline.clone();
        update.version = "1.0.1".into();
        update.image.tag = "1.0.1".into();
        update.image.digest = format!("sha256:{}", "b".repeat(64));
        let raw = image(
            Vec::new(),
            &config,
            &layer,
            br#"[{"Config":"config.json","Layers":["layer.tar"],"RepoTags":null}]"#,
        )?;
        let library = Context {
            home: root.0.join("library"),
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        fs::create_dir(&library.home)?;
        let store = Store::open(&library)?;
        let source = root.0.join("image.gz");
        let (hash, bytes) = save_tar(raw.as_slice(), &source)?;
        let receipt = Receipt::from(&update, hash, bytes);
        let stamp = store.publish_blob(&source, &receipt)?;
        store.write_index(&BTreeMap::from([(
            update.key(),
            Stored {
                service: update.clone(),
                receipt: receipt.clone(),
                stamp,
            },
        )]))?;
        let path = root.0.join("update.devlineinfra");
        export(
            &library,
            &store,
            &[update.clone()],
            &json!({"path":path,"images":["fixture@1.0.1"]}),
            None,
        )?;

        let target = Context {
            home: root.0.join("target"),
            ..library
        };
        fs::create_dir(&target.home)?;
        let target_store = Store::open(&target)?;
        let supported = std::slice::from_ref(&baseline);
        let preview = inspect(supported, &json!({"path":path}))?;
        assert_eq!(preview["requires_trust"], true);
        assert_eq!(preview["images"][0]["known"], false);
        assert!(import(&target_store, supported, &json!({"path":path}), None).is_err());
        assert!(import(
            &target_store,
            supported,
            &json!({"path":path,"trust_images":true,"manifest_sha256":"changed"}),
            None
        )
        .is_err());
        assert!(target_store.index()?.is_empty());
        import(
            &target_store,
            supported,
            &json!({"path":path,"trust_images":true,"manifest_sha256":preview["manifest_sha256"]}),
            None,
        )?;
        assert_eq!(
            target_store.index()?[&update.key()].receipt.image_id,
            receipt.image_id
        );
        Ok(())
    }
}
