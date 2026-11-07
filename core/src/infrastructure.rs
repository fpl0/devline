//! Offline infrastructure images are stored independently of the Docker daemon.
mod archive;
mod docker;
mod registry;
mod stack;

use crate::{tasks::Job, templates, text_arg, Context};
use anyhow::{bail, Context as _, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

const MAX_METADATA: u64 = 8 * 1024 * 1024;
const MAX_IMAGE: u64 = 16 * 1024 * 1024 * 1024;

#[derive(Clone, Deserialize, Serialize)]
struct Image {
    registry: String,
    repository: String,
    tag: String,
    index_digest: Option<String>,
    digest: String,
    config_digest: String,
    platform: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    compressed_bytes: Option<u64>,
}

impl Image {
    fn source(&self) -> String {
        format!("{}/{}@{}", self.registry, self.repository, self.digest)
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Port {
    id: String,
    name: String,
    container_port: u16,
    host_port: u16,
    protocol: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Volume {
    id: String,
    target: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Healthcheck {
    command: Vec<String>,
    interval_seconds: u32,
    timeout_seconds: u32,
    retries: u32,
    #[serde(default)]
    start_period_seconds: u32,
}

#[derive(Clone, Deserialize, Serialize)]
struct Service {
    id: String,
    name: String,
    version: String,
    category: String,
    description: String,
    image: Image,
    #[serde(default)]
    ports: Vec<Port>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    volumes: Vec<Volume>,
    #[serde(default)]
    command: Vec<String>,
    healthcheck: Healthcheck,
    provenance: Value,
    docs_ref: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

impl Service {
    fn key(&self) -> String {
        format!("{}@{}/{}", self.id, self.version, self.image.platform)
    }
    fn alias(&self) -> String {
        format!(
            "devline.local/{}:{}",
            self.id,
            self.image.config_digest.trim_start_matches("sha256:")
        )
    }
}

#[derive(Deserialize)]
struct Catalog {
    schema_version: u32,
    services: Vec<Service>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Receipt {
    id: String,
    version: String,
    platform: String,
    source: String,
    source_digest: String,
    image_id: String,
    archive_sha256: String,
    size_bytes: u64,
    verified_at: u64,
}

impl Receipt {
    fn from(service: &Service, hash: String, bytes: u64) -> Self {
        Self {
            id: service.id.clone(),
            version: service.version.clone(),
            platform: service.image.platform.clone(),
            source: service.image.source(),
            source_digest: service.image.digest.clone(),
            image_id: service.image.config_digest.clone(),
            archive_sha256: hash,
            size_bytes: bytes,
            verified_at: crate::tasks::now(),
        }
    }

    fn validate(&self, service: &Service) -> Result<()> {
        if self.id != service.id
            || self.version != service.version
            || self.platform != service.image.platform
            || self.source != service.image.source()
            || self.source_digest != service.image.digest
            || self.image_id != service.image.config_digest
            || !hash(&self.archive_sha256)
            || !(1..=MAX_IMAGE).contains(&self.size_bytes)
        {
            bail!(
                "Image receipt does not match the pinned {}@{} release",
                service.id,
                service.version
            );
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
struct Stamp {
    size: u64,
    seconds: u64,
    nanos: u32,
    inode: u64,
    #[serde(default)]
    changed_seconds: i64,
    #[serde(default)]
    changed_nanos: i64,
}

impl Stamp {
    fn read(metadata: &Metadata) -> Result<Self> {
        let time = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?;
        #[cfg(unix)]
        let (inode, changed_seconds, changed_nanos) = {
            use std::os::unix::fs::MetadataExt;
            (metadata.ino(), metadata.ctime(), metadata.ctime_nsec())
        };
        #[cfg(not(unix))]
        let (inode, changed_seconds, changed_nanos) = (0, 0, 0);
        Ok(Self {
            size: metadata.len(),
            seconds: time.as_secs(),
            nanos: time.subsec_nanos(),
            inode,
            changed_seconds,
            changed_nanos,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Stored {
    service: Service,
    receipt: Receipt,
    stamp: Stamp,
}

type Index = BTreeMap<String, Stored>;

struct Store {
    root: PathBuf,
    images: PathBuf,
}

impl Store {
    fn open(ctx: &Context) -> Result<Self> {
        Ok(Self {
            root: templates::library(ctx, Path::new("infrastructure"), true)?,
            images: templates::library(ctx, Path::new("infrastructure/images"), true)?,
        })
    }

    fn lock(&self, job: &mut Option<&mut Job>) -> Result<File> {
        let mut options = OpenOptions::new();
        options.create(true).truncate(false).read(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(self.root.join("archive.lock"))?;
        if !file.metadata()?.is_file() {
            bail!("Invalid infrastructure archive lock");
        }
        loop {
            checkpoint(job)?;
            match FileExt::try_lock_exclusive(&file) {
                Ok(()) => return Ok(file),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(75));
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn index(&self) -> Result<Index> {
        let path = self.root.join("index.json");
        if !path.try_exists()? {
            return Ok(Index::new());
        }
        let index: Index = read_json(&path)?;
        if index.len() > 4096 {
            bail!("Infrastructure archive index is too large");
        }
        for (key, item) in &index {
            validate_service(&item.service)?;
            if *key != item.service.key() {
                bail!("Invalid archive index entry");
            }
            item.receipt.validate(&item.service)?;
        }
        Ok(index)
    }

    fn write_index(&self, index: &Index) -> Result<()> {
        write_json_atomic(&self.root, "index.json", index)
    }

    fn blob(&self, receipt: &Receipt) -> PathBuf {
        self.images
            .join(format!("{}.tar.gz", receipt.archive_sha256))
    }

    fn publish_blob(&self, file: &Path, receipt: &Receipt) -> Result<Stamp> {
        let path = self.blob(receipt);
        if fs::symlink_metadata(&path).is_ok() {
            archive::verify_checksum(&path, receipt, &mut None)?;
        } else {
            templates::rename_exclusive(file, &path)?;
            File::open(&self.images)?.sync_all()?;
        }
        Stamp::read(&fs::symlink_metadata(path)?)
    }
}

fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(hash)
}
fn slug(value: &str) -> bool {
    value.len() <= 64
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn version(value: &str) -> bool {
    value.len() <= 64
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
}

fn platform() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "linux/arm64"
    } else {
        "linux/amd64"
    }
}

fn validate_service(service: &Service) -> Result<()> {
    let image = &service.image;
    if !slug(&service.id)
        || !version(&service.version)
        || service.name.is_empty()
        || !matches!(image.platform.as_str(), "linux/arm64" | "linux/amd64")
        || !matches!(image.registry.as_str(), "docker.io" | "quay.io" | "ghcr.io")
        || image.repository.is_empty()
        || image.repository.len() > 128
        || !image.repository.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        })
        || !version(&image.tag)
        || !digest(&image.digest)
        || !digest(&image.config_digest)
        || image
            .index_digest
            .as_deref()
            .is_some_and(|value| !digest(value))
        || service.ports.len() > 16
        || service.volumes.len() > 16
        || service.env.len() > 64
        || service.command.len() > 64
        || service.healthcheck.command.len() > 64
        || !service
            .healthcheck
            .command
            .first()
            .is_some_and(|s| matches!(s.as_str(), "CMD" | "CMD-SHELL"))
        || service.healthcheck.interval_seconds == 0
        || service.healthcheck.timeout_seconds == 0
        || service.healthcheck.retries == 0
    {
        bail!("Invalid infrastructure catalog entry for {}", service.id);
    }
    let mut ports = BTreeSet::new();
    for port in &service.ports {
        if !slug(&port.id)
            || !ports.insert(&port.id)
            || port.host_port == 0
            || port.container_port == 0
            || !matches!(port.protocol.as_str(), "tcp" | "udp")
        {
            bail!("Invalid port configuration for {}", service.id);
        }
    }
    let mut volumes = BTreeSet::new();
    for volume in &service.volumes {
        if !slug(&volume.id)
            || !volumes.insert(&volume.id)
            || !volume.target.starts_with('/')
            || volume.target == "/"
            || volume.target.contains(':')
            || volume
                .target
                .split('/')
                .any(|part| part == ".." || part == ".")
        {
            bail!("Invalid volume configuration for {}", service.id);
        }
    }
    for key in service.env.keys() {
        if key.is_empty()
            || !key.as_bytes()[0].is_ascii_uppercase()
            || !key
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        {
            bail!("Invalid environment name for {}", service.id);
        }
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let mut contents = Vec::new();
    archive::regular(path, MAX_METADATA)
        .with_context(|| format!("Open metadata at {}", path.display()))?
        .take(MAX_METADATA + 1)
        .read_to_end(&mut contents)?;
    if contents.len() as u64 > MAX_METADATA {
        bail!("Metadata exceeds the size limit");
    }
    serde_json::from_slice(&contents)
        .with_context(|| format!("Read metadata at {}", path.display()))
}

fn write_json_atomic(root: &Path, name: &str, value: &impl Serialize) -> Result<()> {
    let stage = templates::Stage::new(root)?;
    let temp = stage.0.join("data");
    let mut file = File::create(&temp)?;
    serde_json::to_writer(&mut file, value)?;
    file.sync_all()?;
    fs::rename(temp, root.join(name))?;
    File::open(root)?.sync_all()?;
    Ok(())
}

fn catalog_services(ctx: &Context, index: &Index) -> Result<Vec<Service>> {
    let path = ctx.catalog_dir.join("infrastructure.json");
    let catalog = if path.try_exists()? {
        read_json::<Catalog>(&path)?
    } else {
        Catalog {
            schema_version: 1,
            services: Vec::new(),
        }
    };
    if catalog.schema_version != 1 || catalog.services.len() > 1024 {
        bail!("Unsupported infrastructure catalog");
    }
    let mut entries = BTreeMap::new();
    for item in catalog.services {
        validate_service(&item)?;
        if entries.insert(item.key(), item).is_some() {
            bail!("Duplicate infrastructure release");
        }
    }
    let help: BTreeMap<_, _> = entries
        .values()
        .filter_map(|item| Some((item.id.clone(), item.extra.get("help_ref")?.clone())))
        .collect();
    for (key, stored) in index {
        if let Some(item) = entries.get(key) {
            stored.receipt.validate(item)?;
        } else {
            let mut service = stored.service.clone();
            // Older archives can contain product help in their reference metadata.
            // Official references apply only to exact catalog releases.
            service.docs_ref = None;
            service.extra.remove("help_ref");
            if let Some(reference) = help.get(&service.id) {
                service.extra.insert("help_ref".into(), reference.clone());
            }
            entries.insert(key.clone(), service);
        }
    }
    Ok(entries.into_values().collect())
}

fn select(services: &[Service], reference: &str) -> Result<Service> {
    let (id, pin) = reference
        .split_once('@')
        .map_or((reference, None), |(id, pin)| (id, Some(pin)));
    if !slug(id) || pin.is_some_and(|pin| !version(pin)) {
        bail!("Use a service ID or a pinned service@version");
    }
    services
        .iter()
        .filter(|service| {
            service.id == id
                && service.image.platform == platform()
                && pin.is_none_or(|pin| service.version == pin)
        })
        .max_by(|a, b| crate::updates::compare_doc_releases(&a.version, &b.version))
        .cloned()
        .with_context(|| {
            format!(
                "No {} infrastructure release matches {reference}",
                platform()
            )
        })
}

fn checkpoint(job: &Option<&mut Job>) -> Result<()> {
    if let Some(job) = job.as_deref() {
        job.checkpoint()?;
    }
    Ok(())
}

fn stage(job: &mut Option<&mut Job>, message: &str) -> Result<()> {
    if let Some(job) = job.as_deref_mut() {
        job.stage(message)?;
    }
    Ok(())
}

fn archive_stamp(path: &Path) -> Result<Stamp> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() {
        bail!("Invalid local image archive");
    }
    Stamp::read(&metadata)
}

fn verify_saved(
    path: &Path,
    service: &Service,
    item: &mut Stored,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    let stamp = archive_stamp(path)?;
    archive::verify_checksum(path, &item.receipt, job)?;
    archive::verify_tar(path, service, job)?;
    if archive_stamp(path)? != stamp {
        bail!("Image archive changed during verification");
    }
    item.stamp = stamp;
    item.receipt.verified_at = crate::tasks::now();
    Ok(())
}

fn image_status(
    store: &Store,
    service: &Service,
    saved: Option<&Stored>,
    docker: Option<&docker::Docker>,
    loaded: bool,
) -> Value {
    let metadata = saved.and_then(|saved| fs::symlink_metadata(store.blob(&saved.receipt)).ok());
    let archived = saved
        .zip(metadata.as_ref())
        .is_some_and(|(entry, meta)| meta.is_file() && meta.len() == entry.receipt.size_bytes);
    let intact = archived
        && saved
            .zip(metadata.as_ref())
            .is_some_and(|(entry, meta)| Stamp::read(meta).ok().as_ref() == Some(&entry.stamp));
    let compatible = docker.is_some_and(|docker| docker.platform() == service.image.platform);
    json!({"id":service.id,"version":service.version,"platform":service.image.platform,
        "archived":archived,"loaded":loaded,"ready_offline":intact && compatible,
        "integrity":if intact { "verified" } else if archived { "changed" } else { "missing" },
        "status":if !archived { "needs_archive" } else if !intact { "needs_verification" }
            else if !compatible { "needs_docker" } else if loaded { "loaded" } else { "ready_offline" },
        "receipt":saved.map(|entry| &entry.receipt)})
}

pub(crate) fn plan_project(ctx: &Context, references: &[String], ports: &Value) -> Result<Value> {
    let empty = serde_json::Map::new();
    let overrides = if ports.is_null() {
        &empty
    } else {
        ports
            .as_object()
            .context("Port overrides must be an object")?
    };
    if references.is_empty() {
        if !overrides.is_empty() {
            bail!("Choose a service before overriding its ports");
        }
        return Ok(Value::Null);
    }
    if references.len() > 16 {
        bail!("Choose at most sixteen services");
    }
    let store = Store {
        root: templates::library(ctx, Path::new("infrastructure"), false)?,
        images: templates::library(ctx, Path::new("infrastructure/images"), false)?,
    };
    let index = store.index()?;
    let catalog = catalog_services(ctx, &index)?;
    let services = references
        .iter()
        .map(|reference| select(&catalog, reference))
        .collect::<Result<Vec<_>>>()?;
    let mut bound = std::collections::BTreeSet::new();
    for (key, value) in overrides {
        let selected = services
            .iter()
            .flat_map(|service| service.ports.iter().map(move |port| (service, port)))
            .find(|(service, port)| format!("{}:{}", service.id, port.id) == *key)
            .with_context(|| {
                format!("Port override {key} refers to a service or port that was not selected")
            })?;
        let port = value
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .filter(|port| *port > 0)
            .with_context(|| format!("Port override {key} must be between 1 and 65535"))?;
        let protocol = selected.1.protocol.as_str();
        if !bound.insert((protocol, port)) || stack::available(port, protocol).is_err() {
            bail!("Host port {port}/{protocol} is already in use");
        }
    }
    let docker = docker::Docker::discover();
    let images = services
        .iter()
        .map(|service| {
            image_status(
                &store,
                service,
                index.get(&service.key()),
                docker.as_ref().ok(),
                false,
            )
        })
        .collect::<Vec<_>>();
    Ok(
        json!({"docker":docker::report(&docker),"platform":platform(),"images":images,"ports":overrides}),
    )
}

pub(crate) fn execute(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let command = text_arg(request, "command")?;
    if command.starts_with("infra.stack.") {
        return stack::execute(ctx, request, job);
    }
    let store = Store::open(ctx)?;
    let index = store.index()?;
    let services = catalog_services(ctx, &index)?;
    match command {
        "infra.catalog" => return Ok(json!({"services":services,"platform":platform()})),
        "infra.status" => {
            let selected = request["id"]
                .as_str()
                .map(|id| select(&services, id))
                .transpose()?;
            let docker = docker::Docker::discover();
            let loaded = docker
                .as_ref()
                .ok()
                .and_then(|docker| docker.images().ok())
                .unwrap_or_default();
            let images: Vec<_> = services
                .iter()
                .filter(|s| {
                    s.image.platform == platform()
                        && selected
                            .as_ref()
                            .is_none_or(|selected| selected.key() == s.key())
                })
                .map(|service| {
                    image_status(
                        &store,
                        service,
                        index.get(&service.key()),
                        docker.as_ref().ok(),
                        loaded.get(&service.alias()) == Some(&service.image.config_digest),
                    )
                })
                .collect();
            return Ok(
                json!({"docker":docker::report(&docker),"platform":platform(),"images":images}),
            );
        }
        "infra.export" => return archive::export(ctx, &store, &services, request, job),
        "infra.import" | "infra.import.inspect" => {
            let mut trusted = registry::available_pins(&store, &services)?;
            trusted.extend(services);
            if command == "infra.import.inspect" {
                return archive::inspect(&trusted, request);
            }
            return archive::import(&store, &trusted, request, job);
        }
        "infra.updates" | "infra.updates.check" => {
            return registry::status(
                &store,
                &services,
                request["id"].as_str(),
                command == "infra.updates.check",
            );
        }
        "infra.update" => return registry::apply(&store, &services, text_arg(request, "id")?, job),
        _ => {}
    }
    let service = select(&services, text_arg(request, "id")?)?;
    match command {
        "infra.prepare" => prepare(&store, &service, job),
        "infra.restore" => restore(&store, &service, job),
        "infra.verify" => {
            stage(&mut job, "Verifying independent image archive")?;
            let _lock = store.lock(&mut job)?;
            let mut index = store.index()?;
            let item = index
                .get_mut(&service.key())
                .context("Prepare this image first")?;
            verify_saved(&store.blob(&item.receipt), &service, item, &mut job)?;
            store.write_index(&index)?;
            let item = index.get(&service.key());
            let mut result = image_status(&store, &service, item, None, false);
            result["archive_verified"] = json!(true);
            result["status"] = json!("verified");
            Ok(result)
        }
        "infra.remove" => {
            let _lock = store.lock(&mut job)?;
            let mut index = store.index()?;
            let removed = index.remove(&service.key());
            if let Some(item) = &removed {
                store.write_index(&index)?;
                if !index
                    .values()
                    .any(|other| other.receipt.archive_sha256 == item.receipt.archive_sha256)
                {
                    fs::remove_file(store.blob(&item.receipt)).or_else(|error| {
                        if error.kind() == io::ErrorKind::NotFound {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    })?;
                }
            }
            Ok(json!({"id":service.id,"version":service.version,"removed":removed.is_some()}))
        }
        other => bail!("Unknown infrastructure command {other}"),
    }
}

fn prepare(store: &Store, service: &Service, mut job: Option<&mut Job>) -> Result<Value> {
    stage(&mut job, "Checking local Docker and archive")?;
    let docker = docker::Docker::discover()?;
    docker.require_platform(&service.image.platform)?;
    let _lock = store.lock(&mut job)?;
    let mut index = store.index()?;
    let key = service.key();
    if let Some(item) = index.get_mut(&key) {
        let path = store.blob(&item.receipt);
        if archive_stamp(&path).ok().as_ref() != Some(&item.stamp) {
            stage(&mut job, "Verifying the existing image archive")?;
            verify_saved(&path, service, item, &mut job)?;
            store.write_index(&index)?;
        }
        return Ok(image_status(
            store,
            service,
            index.get(&key),
            Some(&docker),
            docker.alias_loaded(service)?,
        ));
    }
    let source = service.image.source();
    if !docker.image_matches(&source, service)? {
        docker.run(
            &[
                "image",
                "pull",
                "--platform",
                &service.image.platform,
                &source,
            ],
            "Pulling pinned image",
            &mut job,
        )?;
    }
    if !docker.image_matches(&source, service)? {
        bail!("Docker did not resolve the pinned image identity");
    }
    let stage_dir = templates::Stage::new(&store.images)?;
    let output = stage_dir.0.join("image.tar.gz");
    let (hash, bytes) = docker.save(service, &output, &mut job)?;
    archive::verify_tar(&output, service, &mut job)?;
    checkpoint(&job)?;
    let receipt = Receipt::from(service, hash, bytes);
    let stamp = store.publish_blob(&output, &receipt)?;
    index.insert(
        service.key(),
        Stored {
            service: service.clone(),
            receipt,
            stamp,
        },
    );
    store.write_index(&index)?;
    Ok(image_status(
        store,
        service,
        index.get(&service.key()),
        Some(&docker),
        docker.alias_loaded(service)?,
    ))
}

fn restore(store: &Store, service: &Service, mut job: Option<&mut Job>) -> Result<Value> {
    stage(&mut job, "Checking local Docker and archive")?;
    let docker = docker::Docker::discover()?;
    docker.require_platform(&service.image.platform)?;
    let _lock = store.lock(&mut job)?;
    let index = store.index()?;
    let item = index
        .get(&service.key())
        .context("Prepare or import this image first")?;
    restore_with(store, service, item, &docker, &mut job)?;
    Ok(image_status(
        store,
        service,
        Some(item),
        Some(&docker),
        true,
    ))
}

fn restore_with(
    store: &Store,
    service: &Service,
    item: &Stored,
    docker: &docker::Docker,
    job: &mut Option<&mut Job>,
) -> Result<()> {
    docker.require_platform(&service.image.platform)?;
    let path = store.blob(&item.receipt);
    let loaded = docker.alias_loaded(service)?;
    let stamp = archive_stamp(&path).context("Read the independent image archive")?;
    if loaded && stamp == item.stamp {
        return Ok(());
    }
    stage(job, "Verifying the saved image checksum")?;
    archive::verify_checksum(&path, &item.receipt, job)?;
    if loaded {
        return Ok(());
    }
    if !docker.image_matches(&service.image.config_digest, service)? {
        archive::verify_tar(&path, service, job)?;
        if archive_stamp(&path)? != stamp {
            bail!("Image archive changed during verification");
        }
        let path = path.to_str().context("Archive path is not UTF-8")?;
        docker.run(
            &["image", "load", "--input", path],
            "Loading the local archive",
            job,
        )?;
    }
    if !docker.image_matches(&service.image.config_digest, service)? {
        bail!("Loaded image identity does not match the archive");
    }
    docker.tag(service, job)
}
