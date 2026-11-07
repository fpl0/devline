//! Explicit online checks resolve exact, compatible publisher tags to native image pins.
use super::*;
use sha2::{Digest, Sha256};
use std::process::Command;

const ACCEPT: &str = "Accept: application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json";
const LIMIT: usize = 2 * 1024 * 1024;

#[derive(Clone, Deserialize, Serialize)]
struct Observation {
    checked_at: u64,
    offer: Option<Offer>,
    error: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Offer {
    version: String,
    tag: String,
    index_digest: Option<String>,
    digest: String,
    config_digest: String,
    compressed_bytes: u64,
}

type Observations = BTreeMap<String, Observation>;

fn observations(store: &Store) -> Result<Observations> {
    let path = store.root.join("updates.json");
    if path.try_exists()? {
        read_json(&path)
    } else {
        Ok(BTreeMap::new())
    }
}

fn key(service: &Service) -> String {
    format!("{}/{}", service.id, service.image.platform)
}

fn latest(services: &[Service]) -> BTreeMap<&str, &Service> {
    let mut selected: BTreeMap<&str, &Service> = BTreeMap::new();
    for service in services
        .iter()
        .filter(|service| service.image.platform == platform())
    {
        if selected.get(service.id.as_str()).is_none_or(|current| {
            crate::updates::compare_doc_releases(&service.version, &current.version).is_gt()
        }) {
            selected.insert(&service.id, service);
        }
    }
    selected
}

struct Channel<'a> {
    prefix: &'a str,
    suffix: &'a str,
    branch: String,
    components: usize,
}

impl<'a> Channel<'a> {
    fn new(service: &'a Service) -> Result<Self> {
        let (prefix, suffix) = service
            .image
            .tag
            .split_once(&service.version)
            .context("Image tag does not expose an exact release version")?;
        let components: Vec<_> = service.version.split('.').collect();
        if !components.iter().all(|part| part.parse::<u32>().is_ok()) {
            bail!("Image tag uses an unsupported version scheme");
        }
        let count = if matches!(service.id.as_str(), "mysql" | "mongodb" | "nats") {
            2
        } else {
            1
        };
        let branch = components
            .iter()
            .take(count)
            .copied()
            .collect::<Vec<_>>()
            .join(".");
        Ok(Self {
            prefix,
            suffix,
            branch,
            components: components.len(),
        })
    }

    fn version(&self, tag: &str) -> Option<Vec<u32>> {
        let version = tag.strip_prefix(self.prefix)?.strip_suffix(self.suffix)?;
        if !version.strip_prefix(&self.branch)?.starts_with('.') {
            return None;
        }
        let parts = version
            .split('.')
            .map(|part| {
                if part.is_empty()
                    || !part.bytes().all(|byte| byte.is_ascii_digit())
                    || part.len() > 1 && part.starts_with('0')
                {
                    return None;
                }
                part.parse::<u32>().ok()
            })
            .collect::<Option<Vec<_>>>()?;
        (parts.len() == self.components).then_some(parts)
    }

    fn version_string(&self, tag: &str) -> Option<String> {
        Some(
            self.version(tag)?
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join("."),
        )
    }
}

fn get(url: &str, token: Option<&str>, accept: &str) -> Result<Vec<u8>> {
    let mut command = Command::new("/usr/bin/curl");
    command.args([
        "--disable",
        "--silent",
        "--show-error",
        "--fail",
        "--location",
        "--max-redirs",
        "3",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--connect-timeout",
        "5",
        "--max-time",
        "14",
        "--max-filesize",
        "2097152",
        "--header",
        accept,
    ]);
    if let Some(token) = token {
        command
            .arg("--header")
            .arg(format!("Authorization: Bearer {token}"));
    }
    let result = command
        .arg(url)
        .output()
        .context("Request the publisher registry")?;
    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr);
        bail!(
            "Publisher registry request failed: {}",
            error
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(240)
                .collect::<String>()
        );
    }
    if result.stdout.is_empty() || result.stdout.len() > LIMIT {
        bail!("Invalid publisher registry response size");
    }
    Ok(result.stdout)
}

fn get_json(url: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&get(
        url,
        None,
        "Accept: application/json",
    )?)?)
}

fn tags(service: &Service, channel: &Channel) -> Result<Vec<String>> {
    let repository = &service.image.repository;
    let mut tags = BTreeSet::new();
    for page in 1..=8 {
        let (result, field, more) = match service.image.registry.as_str() {
            "docker.io" => {
                let (namespace, name) = repository
                    .split_once('/')
                    .context("Invalid Docker Hub publisher")?;
                let filter = format!("{}{}.", channel.prefix, channel.branch);
                let value = get_json(&format!("https://hub.docker.com/v2/namespaces/{namespace}/repositories/{name}/tags?page_size=100&page={page}&name={filter}"))?;
                let more = value["next"].is_string();
                (value, "results", more)
            }
            "quay.io" => {
                let value = get_json(&format!("https://quay.io/api/v1/repository/{repository}/tag/?limit=100&page={page}&onlyActiveTags=true"))?;
                let more = value["has_additional"] == true;
                (value, "tags", more)
            }
            _ => bail!("This publisher does not expose a supported stable tag channel"),
        };
        let rows = result[field]
            .as_array()
            .context("Publisher did not return a tag list")?;
        tags.extend(
            rows.iter()
                .filter_map(|row| row["name"].as_str())
                .filter_map(|tag| channel.version(tag).map(|parts| (parts, tag.to_owned()))),
        );
        if !more {
            return Ok(tags.into_iter().rev().map(|(_, tag)| tag).collect());
        }
    }
    bail!("Publisher returned too many tags to verify this release channel");
}

fn content_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn json_digest(bytes: &[u8], expected: Option<&str>) -> Result<Value> {
    if expected.is_some_and(|expected| content_digest(bytes) != expected) {
        bail!("Publisher bytes do not match their OCI digest");
    }
    Ok(serde_json::from_slice(bytes)?)
}

fn version_matches(service: &Service, config: &Value, version: &str) -> bool {
    let env_name = match service.id.as_str() {
        "postgres" => Some("PG_VERSION"),
        "mysql" => Some("MYSQL_VERSION"),
        "rabbitmq" => Some("RABBITMQ_VERSION"),
        "redis" => Some("REDIS_VERSION"),
        "mongodb" => Some("MONGO_VERSION"),
        _ => None,
    };
    let content = &config["config"];
    let declared = if let Some(env_name) = env_name {
        content["Env"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .find_map(|pair| {
                pair.split_once('=')
                    .filter(|(key, _)| *key == env_name)
                    .map(|(_, value)| value)
            })
    } else {
        content["Labels"]["org.opencontainers.image.version"].as_str()
    };
    declared.is_none_or(|value| {
        let value = value.strip_prefix('v').unwrap_or(value);
        value == version
            || service.id == "mysql"
                && value
                    .strip_prefix(version)
                    .is_some_and(|suffix| suffix.starts_with('-'))
    })
}

fn resolve(service: &Service, tag: &str, version: String) -> Result<Option<Offer>> {
    let repository = &service.image.repository;
    let (registry, token) = match service.image.registry.as_str() {
        "docker.io" => {
            let value = get_json(&format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{repository}:pull"))?;
            let token = value["token"]
                .as_str()
                .filter(|token| {
                    token.len() < 16_384
                        && token
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
                })
                .context("Docker Hub did not issue a public pull token")?
                .to_owned();
            ("registry-1.docker.io", Some(token))
        }
        "quay.io" => ("quay.io", None),
        _ => bail!("Unsupported publisher registry"),
    };
    let url = format!("https://{registry}/v2/{repository}");
    let raw = get(&format!("{url}/manifests/{tag}"), token.as_deref(), ACCEPT)?;
    let top = content_digest(&raw);
    let mut manifest = json_digest(&raw, None)?;
    let (os, arch) = service
        .image
        .platform
        .split_once('/')
        .context("Invalid image platform")?;
    let (index_digest, manifest_digest) = if let Some(entries) = manifest["manifests"].as_array() {
        let mut native = entries.iter().filter(|entry| {
            entry["platform"]["os"] == os
                && entry["platform"]["architecture"] == arch
                && entry["platform"]["variant"]
                    .as_str()
                    .is_none_or(|variant| arch == "arm64" && variant == "v8")
        });
        let Some(entry) = native.next() else {
            return Ok(None);
        };
        if native.next().is_some() {
            bail!("Publisher returned ambiguous native image manifests");
        }
        let digest_value = entry["digest"]
            .as_str()
            .filter(|value| digest(value))
            .context("Invalid native image manifest digest")?
            .to_owned();
        let bytes = get(
            &format!("{url}/manifests/{digest_value}"),
            token.as_deref(),
            ACCEPT,
        )?;
        manifest = json_digest(&bytes, Some(&digest_value))?;
        (Some(top), digest_value)
    } else {
        (None, top)
    };
    if manifest["schemaVersion"] != 2 {
        bail!("Unsupported native image manifest");
    }
    let config_digest = manifest["config"]["digest"]
        .as_str()
        .filter(|value| digest(value))
        .context("Missing native image configuration digest")?;
    let bytes = get(
        &format!("{url}/blobs/{config_digest}"),
        token.as_deref(),
        "Accept: application/octet-stream",
    )?;
    let config = json_digest(&bytes, Some(config_digest))?;
    if config["os"] != os || config["architecture"] != arch {
        return Ok(None);
    }
    if !version_matches(service, &config, &version) {
        bail!("Publisher config version differs from the exact image tag");
    }
    let layers = manifest["layers"]
        .as_array()
        .context("Missing native image layers")?;
    let compressed_bytes = layers.iter().try_fold(0_u64, |sum, layer| {
        if !layer["digest"].as_str().is_some_and(digest) {
            bail!("Invalid native layer digest");
        }
        let bytes = layer["size"]
            .as_u64()
            .context("Invalid native layer size")?;
        sum.checked_add(bytes).context("Native image is too large")
    })?;
    if !(1..=MAX_IMAGE).contains(&compressed_bytes) {
        bail!("Native image exceeds the archive limit");
    }
    Ok(Some(Offer {
        version,
        tag: tag.to_owned(),
        index_digest,
        digest: manifest_digest,
        config_digest: config_digest.to_owned(),
        compressed_bytes,
    }))
}

fn check_service(service: &Service) -> Result<Offer> {
    let channel = Channel::new(service)?;
    for tag in tags(service, &channel)?.into_iter().take(8) {
        let version = channel
            .version_string(&tag)
            .context("Invalid exact publisher tag")?;
        if crate::updates::compare_doc_releases(&version, &service.version).is_lt() {
            break;
        }
        if let Some(offer) = resolve(service, &tag, version)? {
            return Ok(offer);
        }
    }
    bail!("Publisher did not return a compatible native image for this channel");
}

fn with_offer(current: &Service, offer: &Offer) -> Result<Service> {
    let channel = Channel::new(current)?;
    if channel.version_string(&offer.tag).as_deref() != Some(&offer.version) {
        bail!("Stored image update no longer matches the publisher channel");
    }
    let mut service = current.clone();
    for field in ["source_url", "license_url"] {
        if let Some(url) = service.provenance[field].as_str() {
            service.provenance[field] = json!(url.replace(&current.version, &offer.version));
        }
    }
    service.version = offer.version.clone();
    service.image.tag = offer.tag.clone();
    service.image.index_digest = offer.index_digest.clone();
    service.image.digest = offer.digest.clone();
    service.image.config_digest = offer.config_digest.clone();
    service.image.compressed_bytes = Some(offer.compressed_bytes);
    validate_service(&service)?;
    Ok(service)
}

pub(super) fn available_pins(store: &Store, services: &[Service]) -> Result<Vec<Service>> {
    let cache = observations(store)?;
    Ok(latest(services)
        .into_values()
        .filter_map(|service| {
            let offer = cache.get(&key(service))?.offer.as_ref()?;
            with_offer(service, offer).ok()
        })
        .collect())
}

pub(super) fn transferred_pin(
    services: &[Service],
    receipt: &Receipt,
    image: &Image,
) -> Result<Service> {
    let current = latest(services)
        .remove(receipt.id.as_str())
        .context("No curated service matches this publisher pin")?;
    if image.registry != current.image.registry
        || image.repository != current.image.repository
        || image.platform != current.image.platform
    {
        bail!("The archive publisher or platform differs from the curated service");
    }
    let offer = Offer {
        version: receipt.version.clone(),
        tag: image.tag.clone(),
        index_digest: image.index_digest.clone(),
        digest: image.digest.clone(),
        config_digest: image.config_digest.clone(),
        compressed_bytes: image.compressed_bytes.unwrap_or(0),
    };
    let mut service = with_offer(current, &offer)?;
    service.image.compressed_bytes = image.compressed_bytes;
    receipt.validate(&service)?;
    Ok(service)
}

pub(super) fn status(
    store: &Store,
    services: &[Service],
    id: Option<&str>,
    check: bool,
) -> Result<Value> {
    let selected = latest(services);
    let selected: Vec<_> = if let Some(id) = id {
        vec![*selected
            .get(id)
            .with_context(|| format!("Unknown infrastructure publisher {id}"))?]
    } else {
        selected.into_values().collect()
    };
    if check {
        let checked_at = crate::tasks::now();
        let found = std::thread::scope(|scope| -> Result<Vec<_>> {
            let workers: Vec<_> = selected
                .iter()
                .map(|service| {
                    scope.spawn(move || {
                        let result = check_service(service);
                        (key(service), result)
                    })
                })
                .collect();
            workers
                .into_iter()
                .map(|worker| {
                    worker
                        .join()
                        .map_err(|_| anyhow::anyhow!("Publisher check interrupted"))
                })
                .collect()
        })?;
        let _lock = store.lock(&mut None)?;
        let mut cached = observations(store)?;
        for (key, result) in found {
            if cached
                .get(&key)
                .is_some_and(|old| old.checked_at > checked_at)
            {
                continue;
            }
            let entry = match result {
                Ok(offer) => Observation {
                    checked_at,
                    offer: Some(offer),
                    error: None,
                },
                Err(error) => Observation {
                    checked_at,
                    offer: cached.get(&key).and_then(|entry| entry.offer.clone()),
                    error: Some(format!("{error:#}")),
                },
            };
            cached.insert(key, entry);
        }
        write_json_atomic(&store.root, "updates.json", &cached)?;
    }
    let cache = observations(store)?;
    let mut checked_at = None;
    let items: Vec<_> = selected.into_iter().map(|service| {
        let observation = cache.get(&key(service));
        if let Some(observation) = observation { checked_at = checked_at.max(Some(observation.checked_at)); }
        let offer = observation.and_then(|observation|observation.offer.as_ref())
            .filter(|offer|with_offer(service,offer).is_ok()
                && !crate::updates::compare_doc_releases(&offer.version,&service.version).is_lt());
        let newer = offer.is_some_and(|offer|crate::updates::compare_doc_releases(&offer.version,&service.version).is_gt());
        let republished = offer.is_some_and(|offer|offer.version == service.version
            && (offer.digest != service.image.digest || offer.config_digest != service.image.config_digest));
        let (state,detail) = if let Some(error) = observation.and_then(|entry|entry.error.as_deref()) {
            ("error",error)
        } else if newer {
            ("available","Archive this verified publisher release alongside the current version.")
        } else if republished {
            ("republished","The publisher changed an existing version tag. The installed pin was preserved.")
        } else if offer.is_some() {
            ("up_to_date","The latest compatible stable image matches the current release.")
        } else {
            ("unchecked","Check the publisher for a stable image on this platform.")
        };
        json!({"id":service.id,"name":service.name,"current_version":service.version,
            "available_version":offer.map(|offer|&offer.version),"state":state,"detail":detail,
            "reference":offer.filter(|_|newer && state == "available").map(|offer|format!("{}@{}",service.id,offer.version)),
            "checked_at":observation.map(|entry|entry.checked_at)})
    }).collect();
    Ok(json!({"checked_at":checked_at,"items":items}))
}

pub(super) fn apply(
    store: &Store,
    services: &[Service],
    reference: &str,
    job: Option<&mut Job>,
) -> Result<Value> {
    let (id, expected) = reference
        .split_once('@')
        .map_or((reference, None), |(id, version)| (id, Some(version)));
    let service = latest(services)
        .remove(id)
        .with_context(|| format!("Unknown infrastructure publisher {id}"))?;
    let cache = observations(store)?;
    let observation = cache
        .get(&key(service))
        .filter(|entry| entry.error.is_none())
        .context("Check the publisher for updates first")?;
    let offer = observation
        .offer
        .as_ref()
        .context("No verified publisher release is available")?;
    if expected.is_some_and(|expected| expected != offer.version) {
        bail!("The offered image version changed. Check updates again.");
    }
    if !crate::updates::compare_doc_releases(&offer.version, &service.version).is_gt() {
        bail!("No newer compatible publisher image is available");
    }
    prepare(store, &with_offer(service, offer)?, job)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_stable_tags_keep_compatible_branch_and_variant() -> Result<()> {
        let catalog: Catalog =
            serde_json::from_slice(include_bytes!("../../../data/infrastructure.json"))?;
        let postgres = catalog
            .services
            .iter()
            .find(|service| service.id == "postgres")
            .unwrap();
        let channel = Channel::new(postgres)?;
        assert_eq!(
            channel.version_string("18.10-alpine").as_deref(),
            Some("18.10")
        );
        for invalid in [
            "19.1-alpine",
            "18.10",
            "18.10-alpine3.25",
            "18.10-rc1-alpine",
            "18.010-alpine",
        ] {
            assert!(channel.version(invalid).is_none(), "{invalid}");
        }
        let mysql = catalog
            .services
            .iter()
            .find(|service| service.id == "mysql")
            .unwrap();
        assert!(Channel::new(mysql)?.version("8.5.1").is_none());
        assert!(Channel::new(mysql)?.version("8.4.12").is_some());
        assert!(json_digest(b"{}", Some(&format!("sha256:{}", "0".repeat(64)))).is_err());
        Ok(())
    }
}
