use crate::{lock_library, safe_catalog_path, templates, Context};
use anyhow::{bail, Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const DEFAULT_APP_FEED: &str =
    "https://github.com/fpl0/devline/releases/latest/download/release.json";

#[derive(Deserialize, Serialize)]
struct Settings {
    app_feed: Option<String>,
    checked_at: Option<u64>,
    check_started_at: Option<u64>,
    #[serde(default)]
    app_checked_at: Option<u64>,
    #[serde(default)]
    app_check_started_at: Option<u64>,
    observations: BTreeMap<String, Observation>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_feed: Some(DEFAULT_APP_FEED.to_owned()),
            checked_at: None,
            check_started_at: None,
            app_checked_at: None,
            app_check_started_at: None,
            observations: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Observation {
    version: Option<String>,
    compatible: Option<String>,
    url: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct UpdateItem {
    id: String,
    kind: &'static str,
    name: String,
    current_version: Option<String>,
    available_version: Option<String>,
    state: &'static str,
    detail: String,
    action: Option<&'static str>,
    reference: Option<String>,
    url: Option<String>,
}

impl UpdateItem {
    fn new(kind: &'static str, id: &str, name: &str) -> Self {
        Self {
            id: format!("{kind}:{id}"),
            kind,
            name: name.to_owned(),
            current_version: None,
            available_version: None,
            state: "unchecked",
            detail: String::new(),
            action: None,
            reference: None,
            url: None,
        }
    }

    fn open(&mut self, url: &str) {
        self.action = Some("open_url");
        self.reference = None;
        self.url = Some(url.to_owned());
    }
}

fn settings(ctx: &Context) -> Result<Settings> {
    match fs::read(ctx.home.join("updates.json")) {
        Ok(bytes) => serde_json::from_slice(&bytes).context("Read update settings"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(error) => Err(error).context("Read update settings"),
    }
}

// Callers hold the library lock while writing so configuration and check results
// cannot overwrite each other. Replace the complete snapshot in one rename.
fn save(ctx: &Context, settings: &Settings) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(settings)?;
    let temp = ctx.home.join(format!(
        ".updates-{}-{}.tmp",
        std::process::id(),
        now().as_nanos()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = file.write_all(&bytes).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = result.and_then(|()| fs::rename(&temp, ctx.home.join("updates.json"))) {
        let _ = fs::remove_file(&temp);
        return Err(error).context("Save update settings");
    }
    Ok(())
}

fn now() -> std::time::Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

fn https_url(value: &str) -> Result<()> {
    let authority = value
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .context("Enter an HTTPS release manifest URL")?;
    let (host, port) = authority
        .split_once(':')
        .map_or((authority, None), |(h, p)| (h, Some(p)));
    if host.is_empty()
        || host.split('.').any(|label| {
            label.is_empty()
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        || port.is_some_and(|p| !p.parse::<u16>().is_ok_and(|n| n > 0))
        || value.len() > 2048
        || value
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace() || matches!(ch, '?' | '#' | '\\'))
    {
        bail!("Use an HTTPS URL with a hostname and no credentials, query, or fragment.");
    }
    Ok(())
}

pub fn configure(ctx: &Context, url: &str) -> Result<Value> {
    let url = url.trim();
    if !url.is_empty() {
        https_url(url)?;
    }
    let _lock = lock_library(ctx)?;
    let mut prefs = settings(ctx)?;
    let next = (!url.is_empty()).then(|| url.to_owned());
    if prefs.app_feed != next {
        prefs.app_feed = next;
        prefs.observations.remove("app");
        prefs.app_checked_at = None;
        prefs.app_check_started_at = None;
        save(ctx, &prefs)?;
    }
    drop(_lock);
    make_status(ctx, prefs)
}

#[derive(Clone, Copy)]
enum Feed {
    Go,
    Python,
    Rust,
    Ruby,
    Dotnet,
    Java,
    Maven,
    Elixir,
    Erlang,
    Chi,
    FastApi,
    Axum,
    Tauri,
    Spring,
    Phoenix,
    EntityFramework,
    Rails,
    Ecto,
    Node,
    React,
    Vite,
    TypeScript,
    Playwright,
    Fastify,
}

impl Feed {
    const ALL: [Self; 24] = [
        Self::Go,
        Self::Python,
        Self::Rust,
        Self::Ruby,
        Self::Dotnet,
        Self::Java,
        Self::Maven,
        Self::Elixir,
        Self::Erlang,
        Self::Chi,
        Self::FastApi,
        Self::Axum,
        Self::Tauri,
        Self::Spring,
        Self::Phoenix,
        Self::EntityFramework,
        Self::Rails,
        Self::Ecto,
        Self::Node,
        Self::React,
        Self::Vite,
        Self::TypeScript,
        Self::Playwright,
        Self::Fastify,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::Go => "go",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::Ruby => "ruby",
            Self::Dotnet => "dotnet",
            Self::Java => "java",
            Self::Maven => "maven",
            Self::Elixir => "elixir",
            Self::Erlang => "erlang",
            Self::Chi => "chi",
            Self::FastApi => "fastapi",
            Self::Axum => "axum",
            Self::Tauri => "tauri",
            Self::Spring => "spring-boot",
            Self::Phoenix => "phoenix",
            Self::EntityFramework => "entity-framework",
            Self::Rails => "rails",
            Self::Ecto => "ecto",
            Self::Node => "node",
            Self::React => "react",
            Self::Vite => "vite",
            Self::TypeScript => "typescript",
            Self::Playwright => "playwright",
            Self::Fastify => "fastify",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Go => "Go",
            Self::Python => "CPython",
            Self::Rust => "Rust",
            Self::Ruby => "Ruby",
            Self::Dotnet => ".NET SDK",
            Self::Java => "OpenJDK (Temurin)",
            Self::Maven => "Apache Maven",
            Self::Elixir => "Elixir",
            Self::Erlang => "Erlang/OTP",
            Self::Chi => "Chi",
            Self::FastApi => "FastAPI",
            Self::Axum => "Axum",
            Self::Tauri => "Tauri",
            Self::Spring => "Spring Boot",
            Self::Phoenix => "Phoenix",
            Self::EntityFramework => "Entity Framework Core SQLite",
            Self::Rails => "Ruby on Rails",
            Self::Ecto => "Ecto",
            Self::Node => "Node.js",
            Self::React => "React",
            Self::Vite => "Vite",
            Self::TypeScript => "TypeScript",
            Self::Playwright => "Playwright",
            Self::Fastify => "Fastify",
        }
    }

    fn metadata(self) -> &'static str {
        match self {
            Self::Go => "https://go.dev/dl/?mode=json",
            Self::Python => "https://www.python.org/downloads/macos/",
            Self::Rust => "https://static.rust-lang.org/dist/channel-rust-stable.toml",
            Self::Ruby => "https://www.ruby-lang.org/en/downloads/",
            Self::Dotnet => "https://builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json",
            Self::Java => "https://api.adoptium.net/v3/info/available_releases",
            Self::Maven => "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml",
            Self::Elixir => "https://api.github.com/repos/elixir-lang/elixir/releases/latest",
            Self::Erlang => "https://api.github.com/repos/erlang/otp/releases/latest",
            Self::Chi => "https://proxy.golang.org/github.com/go-chi/chi/v5/@latest",
            Self::FastApi => "https://pypi.org/pypi/fastapi/json",
            Self::Axum => "https://crates.io/api/v1/crates/axum",
            Self::Tauri => "https://crates.io/api/v1/crates/tauri",
            Self::Spring => "https://repo.maven.apache.org/maven2/org/springframework/boot/spring-boot-starter-parent/maven-metadata.xml",
            Self::Phoenix => "https://hex.pm/api/packages/phoenix",
            Self::EntityFramework => "https://api.nuget.org/v3-flatcontainer/microsoft.entityframeworkcore.sqlite/index.json",
            Self::Rails => "https://rubygems.org/api/v1/versions/rails/latest.json",
            Self::Ecto => "https://hex.pm/api/packages/ecto",
            Self::Node => "https://nodejs.org/dist/index.json",
            Self::React => "https://registry.npmjs.org/react/latest",
            Self::Vite => "https://registry.npmjs.org/vite/latest",
            Self::TypeScript => "https://registry.npmjs.org/typescript/latest",
            Self::Playwright => "https://registry.npmjs.org/playwright/latest",
            Self::Fastify => "https://registry.npmjs.org/fastify/latest",
        }
    }

    fn page(self) -> &'static str {
        match self {
            Self::Go => "https://go.dev/dl/",
            Self::Python => "https://www.python.org/downloads/macos/",
            Self::Rust => "https://www.rust-lang.org/tools/install",
            Self::Ruby => "https://www.ruby-lang.org/en/downloads/",
            Self::Dotnet => "https://dotnet.microsoft.com/en-us/download/dotnet",
            Self::Java => "https://adoptium.net/temurin/releases/",
            Self::Maven => "https://maven.apache.org/download.cgi",
            Self::Elixir => "https://elixir-lang.org/install.html",
            Self::Erlang => "https://www.erlang.org/downloads",
            Self::Chi => "https://pkg.go.dev/github.com/go-chi/chi/v5",
            Self::FastApi => "https://fastapi.tiangolo.com/",
            Self::Axum => "https://docs.rs/axum/latest/axum/",
            Self::Tauri => "https://v2.tauri.app/",
            Self::Spring => "https://docs.spring.io/spring-boot/",
            Self::Phoenix => "https://hexdocs.pm/phoenix/",
            Self::EntityFramework => {
                "https://www.nuget.org/packages/Microsoft.EntityFrameworkCore.Sqlite"
            }
            Self::Rails => "https://guides.rubyonrails.org/",
            Self::Ecto => "https://hexdocs.pm/ecto/",
            Self::Node => "https://nodejs.org/en/download",
            Self::React => "https://react.dev/",
            Self::Vite => "https://vite.dev/",
            Self::TypeScript => "https://www.typescriptlang.org/",
            Self::Playwright => "https://playwright.dev/",
            Self::Fastify => "https://fastify.dev/",
        }
    }

    fn compatible(self, parts: &[u64]) -> bool {
        let major = parts[0];
        let minor = parts.get(1).copied().unwrap_or(0);
        // Reflect the compatibility range in each bundled build-latest recipe.
        match self {
            Self::FastApi => major == 0,
            Self::Axum => major == 0 && minor == 8,
            Self::Tauri => major == 2,
            Self::Spring => major == 4 && minor == 1,
            Self::Phoenix => major == 1 && minor == 8,
            Self::EntityFramework => major == 10 && minor == 0,
            _ => true,
        }
    }
}

fn stable_version(value: &str) -> Option<Vec<u64>> {
    let mut numbers: Vec<u64> = value
        .split('.')
        .map(|part| {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                None
            } else {
                part.parse().ok()
            }
        })
        .collect::<Option<_>>()?;
    while numbers.len() > 1 && numbers.last() == Some(&0) {
        numbers.pop();
    }
    Some(numbers)
}

fn newer(candidate: &str, current: &str) -> bool {
    match (stable_version(candidate), stable_version(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => false,
    }
}

pub(super) fn doc_release(version: &str) -> (&str, u64) {
    version
        .split_once("+devline.")
        .and_then(|(upstream, revision)| Some((upstream, revision.parse().ok()?)))
        .unwrap_or((version, 0))
}

pub(super) fn compare_doc_releases(a: &str, b: &str) -> std::cmp::Ordering {
    let (a_base, a_revision) = doc_release(a);
    let (b_base, b_revision) = doc_release(b);
    match (stable_version(a_base), stable_version(b_base)) {
        (Some(a), Some(b)) => a.cmp(&b).then(a_revision.cmp(&b_revision)),
        _ => a_base.cmp(b_base).then(a_revision.cmp(&b_revision)),
    }
    .then(a.cmp(b))
}

fn latest<'a>(versions: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    versions
        .filter_map(|version| Some((stable_version(version)?, version)))
        .max_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(b.1)))
        .map(|(_, version)| version)
}

fn xml_versions(body: &str) -> impl Iterator<Item = &str> {
    body.split("<version>")
        .skip(1)
        .filter_map(|chunk| chunk.split_once("</version>").map(|(v, _)| v.trim()))
}

fn number_at_start(text: &str) -> Option<&str> {
    let text = text.trim_start();
    let end = text
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .unwrap_or(text.len());
    let version = text[..end].trim_end_matches('.');
    stable_version(version).map(|_| version)
}

fn html_text(value: &str) -> String {
    let mut in_tag = false;
    value
        .chars()
        .filter(|&ch| match ch {
            '<' => {
                in_tag = true;
                false
            }
            '>' => {
                in_tag = false;
                false
            }
            _ => !in_tag,
        })
        .collect()
}

fn feed_versions(feed: Feed, body: &str) -> Result<Vec<String>> {
    let one = |value: Option<&str>| {
        value
            .filter(|version| stable_version(version).is_some())
            .map(|version| vec![version.to_owned()])
            .context("Stable release version was absent")
    };
    match feed {
        Feed::Python => {
            let text = html_text(body);
            one(text
                .split_once("Latest Python 3 Release - Python ")
                .and_then(|(_, tail)| number_at_start(tail)))
        }
        Feed::Ruby => {
            let text = html_text(body);
            one(text
                .split_once("The current stable version is ")
                .and_then(|(_, tail)| number_at_start(tail)))
        }
        Feed::Rust => one(body
            .split_once("[pkg.rust]")
            .and_then(|(_, tail)| {
                tail.lines()
                    .find_map(|line| line.strip_prefix("version = \""))
            })
            .and_then(number_at_start)),
        Feed::Maven | Feed::Spring => Ok(xml_versions(body).map(str::to_owned).collect()),
        _ => {
            let json: Value = serde_json::from_str(body).context("Read release metadata")?;
            let many = |value: &Value, field: &str| -> Vec<String> {
                value
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|row| row[field].as_str().map(str::to_owned))
                    .collect()
            };
            match feed {
                Feed::Go => Ok(json
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|entry| entry["stable"] == true)
                    .filter_map(|entry| {
                        entry["version"]
                            .as_str()?
                            .strip_prefix("go")
                            .map(str::to_owned)
                    })
                    .collect()),
                Feed::Node => Ok(json
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|entry| {
                        entry["version"]
                            .as_str()?
                            .strip_prefix('v')
                            .filter(|version| stable_version(version).is_some())
                            .map(str::to_owned)
                    })
                    .collect()),
                Feed::React | Feed::Vite | Feed::TypeScript | Feed::Playwright | Feed::Fastify => {
                    one(json["version"].as_str())
                }
                Feed::Dotnet => Ok(many(&json["releases-index"], "latest-sdk")),
                Feed::Java => json["most_recent_feature_release"]
                    .as_u64()
                    .map(|version| vec![version.to_string()])
                    .context("Latest GA JDK was absent"),
                Feed::Elixir => one(json["tag_name"]
                    .as_str()
                    .and_then(|tag| tag.strip_prefix('v'))),
                Feed::Erlang => one(json["tag_name"]
                    .as_str()
                    .and_then(|tag| tag.strip_prefix("OTP-"))),
                Feed::Chi => one(json["Version"]
                    .as_str()
                    .and_then(|tag| tag.strip_prefix('v'))),
                Feed::FastApi => {
                    let mut versions: Vec<_> = json["releases"]
                        .as_object()
                        .into_iter()
                        .flatten()
                        .filter(|(_, assets)| {
                            assets
                                .as_array()
                                .is_some_and(|a| a.iter().any(|f| f["yanked"] != true))
                        })
                        .map(|(version, _)| version.clone())
                        .collect();
                    versions.extend(json["info"]["version"].as_str().map(str::to_owned));
                    Ok(versions)
                }
                Feed::Axum | Feed::Tauri => Ok(many(&json["versions"], "num")
                    .into_iter()
                    .filter(|version| {
                        json["versions"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .any(|row| row["num"] == *version && row["yanked"] != true)
                    })
                    .collect()),
                Feed::Phoenix | Feed::Ecto => Ok(many(&json["releases"], "version")),
                Feed::EntityFramework => Ok(json["versions"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|row| row.as_str().map(str::to_owned))
                    .collect()),
                Feed::Rails => one(json["version"].as_str()),
                _ => unreachable!("Text feeds handled above"),
            }
        }
    }
}

fn fetch(url: &str) -> Result<String> {
    let output = Command::new("/usr/bin/curl")
        .args([
            "--disable",
            "--globoff",
            "--fail",
            "--silent",
            "--show-error",
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
            "12",
            "--max-filesize",
            "8388608",
            "--user-agent",
            concat!("Devline/", env!("CARGO_PKG_VERSION"), " release checker"),
            "--url",
            url,
        ])
        .output()
        .context("Start HTTPS release check")?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        bail!(
            "{}",
            detail
                .lines()
                .next()
                .unwrap_or("HTTPS request failed")
                .chars()
                .take(220)
                .collect::<String>()
        );
    }
    String::from_utf8(output.stdout).context("Release metadata is not UTF-8")
}

fn observe(feed: Feed) -> Observation {
    let result = fetch(feed.metadata()).and_then(|body| {
        let versions = feed_versions(feed, &body)?;
        let version = latest(versions.iter().map(String::as_str))
            .context("No stable release version in the official metadata")?
            .to_owned();
        let compatible = versions
            .iter()
            .map(String::as_str)
            .filter(|v| stable_version(v).is_some_and(|parts| feed.compatible(&parts)));
        Ok((version, latest_version(compatible)))
    });
    match result {
        Ok((version, compatible)) => Observation {
            version: Some(version),
            compatible,
            url: None,
            error: None,
        },
        Err(error) => Observation {
            version: None,
            compatible: None,
            url: None,
            error: Some(format!("{error:#}")),
        },
    }
}

fn latest_version<'a>(versions: impl Iterator<Item = &'a str>) -> Option<String> {
    latest(versions).map(str::to_owned)
}

#[derive(Deserialize)]
struct AppManifest {
    schema_version: u32,
    version: String,
    url: String,
}

fn parse_app_manifest(body: &str) -> Result<AppManifest> {
    let manifest: AppManifest = serde_json::from_str(body).context("Read app release manifest")?;
    if manifest.schema_version != 1 || stable_version(&manifest.version).is_none() {
        bail!("Expected schema_version 1 and a stable dotted numeric version.");
    }
    https_url(&manifest.url)?;
    Ok(manifest)
}

fn observe_app(url: &str) -> Observation {
    match https_url(url)
        .and_then(|()| fetch(url))
        .and_then(|body| parse_app_manifest(&body))
    {
        Ok(manifest) => Observation {
            version: Some(manifest.version),
            compatible: None,
            url: Some(manifest.url),
            error: None,
        },
        Err(error) => Observation {
            version: None,
            compatible: None,
            url: None,
            error: Some(format!("{error:#}")),
        },
    }
}

fn value_text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_default()
}

fn append_detail(item: &mut UpdateItem, sentence: &str) {
    if !item.detail.is_empty() {
        item.detail.push(' ');
    }
    item.detail.push_str(sentence);
}

fn docs_updates(ctx: &Context, prefs: &Settings) -> Result<Vec<UpdateItem>> {
    let db = ctx.database()?;
    let mut installed = BTreeMap::<String, Vec<String>>::new();
    let mut stmt = db.prepare("SELECT id,version FROM packs ORDER BY id,version")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        installed.entry(row.get(0)?).or_default().push(row.get(1)?);
    }
    let mut updates = Vec::new();
    for (id, versions) in installed {
        let head = ctx
            .catalog
            .docs
            .iter()
            .filter(|p| p.id == id)
            .max_by(|a, b| compare_doc_releases(&a.version, &b.version));
        let mut item = UpdateItem::new("docs", &id, head.map_or(id.as_str(), |p| p.name.as_str()));
        let current = versions
            .iter()
            .max_by(|a, b| compare_doc_releases(a, b))
            .unwrap();
        if id == "swiftui" {
            let local = crate::apple_docs::status(ctx)?;
            item.name = "SwiftUI SDK Reference".into();
            item.current_version = local["active_version"]
                .as_str()
                .map(str::to_owned)
                .or_else(|| Some(current.to_owned()));
            item.detail = local["detail"]
                .as_str()
                .unwrap_or("Imported from this Mac’s selected Xcode SDK.")
                .into();
            if let Some(version) = local["version"]
                .as_str()
                .filter(|_| local["available"] == true)
            {
                if item.current_version.as_deref() == Some(version) {
                    item.state = "current";
                } else {
                    item.state = "available";
                    item.available_version = Some(version.into());
                    item.action = Some("install_docs");
                    item.reference = Some(format!("swiftui@{version}"));
                    append_detail(&mut item, "Prepare the reference from the selected SDK. Earlier imported versions remain available.");
                }
            } else {
                append_detail(&mut item, "The previously imported reference remains available offline. Select an installed Xcode SDK to refresh it.");
            }
            updates.push(item);
            continue;
        }
        let (upstream_version, _) = doc_release(current);
        item.current_version = Some(current.to_owned());
        item.detail = format!("Installed versions: {}.", versions.join(", "));
        let Some(head) = head else {
            append_detail(
                &mut item,
                "This pack has no release in the bundled catalog.",
            );
            updates.push(item);
            continue;
        };
        if compare_doc_releases(&head.version, current).is_gt() {
            item.state = "available";
            item.available_version = Some(head.version.clone());
            item.action = Some("install_docs");
            item.reference = Some(format!("{id}@{}", head.version));
            append_detail(
                &mut item,
                "A newer offline pack is ready to install from this catalog.",
            );
        } else {
            append_detail(&mut item, "Current in the bundled catalog.");
            if let Some(feed) = Feed::ALL.into_iter().find(|feed| feed.id() == id) {
                match prefs.observations.get(&id) {
                    Some(observation) if observation.error.is_some() => {
                        item.state = "error";
                        append_detail(
                            &mut item,
                            &format!(
                                "Upstream version check failed: {}",
                                observation.error.as_deref().unwrap_or_default()
                            ),
                        );
                        if let Some(version) = &observation.version {
                            item.available_version =
                                newer(version, upstream_version).then(|| version.to_owned());
                            append_detail(
                                &mut item,
                                &format!("Last successful upstream result: {version}."),
                            );
                        }
                        item.open(feed.page());
                    }
                    Some(observation) => {
                        if let Some(upstream) = observation
                            .version
                            .as_deref()
                            .filter(|version| newer(version, upstream_version))
                        {
                            item.state = "available";
                            item.available_version = Some(upstream.to_owned());
                            append_detail(
                                &mut item,
                                &format!(
                                    "Upstream {} {upstream} is newer than this offline pack.",
                                    feed.name()
                                ),
                            );
                            let page = if matches!(
                                feed,
                                Feed::Go
                                    | Feed::Python
                                    | Feed::Rust
                                    | Feed::Ruby
                                    | Feed::Java
                                    | Feed::Elixir
                            ) {
                                match feed {
                                    Feed::Go => "https://go.dev/doc/",
                                    Feed::Python => "https://docs.python.org/3/",
                                    Feed::Rust => "https://doc.rust-lang.org/stable/",
                                    Feed::Ruby => "https://docs.ruby-lang.org/",
                                    Feed::Java => "https://docs.oracle.com/en/java/javase/",
                                    Feed::Elixir => "https://hexdocs.pm/elixir/",
                                    _ => unreachable!(),
                                }
                            } else {
                                feed.page()
                            };
                            item.open(page);
                        } else if observation.version.is_some() {
                            item.state = "current";
                            append_detail(
                                &mut item,
                                "The upstream release feed reports no newer version.",
                            );
                        }
                    }
                    None => append_detail(&mut item, "Upstream version has not been checked."),
                }
            } else {
                append_detail(&mut item, "This documentation uses catalog snapshots.");
            }
        }
        updates.push(item);
    }
    Ok(updates)
}

fn starter_feed(id: &str) -> Option<Feed> {
    match id {
        "go/chi-api" | "go/chi-taskboard" | "go/service-kit" => Some(Feed::Chi),
        "python/fastapi-api" | "python/service-kit" => Some(Feed::FastApi),
        "rust/axum-api" | "rust/service-kit" => Some(Feed::Axum),
        "rust/tauri-desktop" => Some(Feed::Tauri),
        "java/spring-api" => Some(Feed::Spring),
        "elixir/phoenix-api" => Some(Feed::Phoenix),
        "dotnet/aspnet-api" => Some(Feed::EntityFramework),
        "node/react-web" => Some(Feed::React),
        "node/playwright-automation" => Some(Feed::Playwright),
        "node/service-kit" => Some(Feed::Fastify),
        _ => None,
    }
}

fn dependency_version(feed: Feed, project: &Path) -> Option<String> {
    let npm = match feed {
        Feed::React => Some("react"),
        Feed::Playwright => Some("@playwright/test"),
        Feed::Fastify => Some("fastify"),
        _ => None,
    };
    if let Some(package) = npm {
        let manifest: Value =
            serde_json::from_slice(&fs::read(project.join("package.json")).ok()?).ok()?;
        let version = manifest["dependencies"][package]
            .as_str()
            .or_else(|| manifest["devDependencies"][package].as_str())?;
        return stable_version(version).map(|_| version.to_owned());
    }
    let (file, marker) = match feed {
        Feed::Chi => ("go.mod", "github.com/go-chi/chi/v5 v"),
        Feed::FastApi => ("requirements.txt", "fastapi=="),
        Feed::Axum => ("Cargo.lock", "name = \"axum\""),
        Feed::Tauri => ("Cargo.lock", "name = \"tauri\""),
        Feed::Spring => (
            "pom.xml",
            "<artifactId>spring-boot-starter-parent</artifactId>",
        ),
        Feed::Phoenix => ("vendor/phoenix/mix.exs", "@version \""),
        Feed::EntityFramework => (
            "OfflineApi.csproj",
            "Include=\"Microsoft.EntityFrameworkCore.Sqlite\" Version=\"[",
        ),
        _ => return None,
    };
    let contents = fs::read_to_string(project.join(file)).ok()?;
    let (_, tail) = contents.split_once(marker)?;
    let value = match feed {
        Feed::Axum | Feed::Tauri => tail.trim_start().strip_prefix("version = \"")?,
        Feed::Spring => tail.trim_start().strip_prefix("<version>")?,
        _ => tail,
    };
    number_at_start(value).map(str::to_owned)
}

fn template_updates(ctx: &Context, prefs: &Settings) -> Result<Vec<UpdateItem>> {
    let catalog = templates::list(ctx)?;
    let mut releases = BTreeMap::<&str, Vec<&Value>>::new();
    for row in catalog.as_array().context("Read template releases")? {
        releases.entry(value_text(row, "id")).or_default().push(row);
    }
    let mut updates = Vec::new();
    for (id, entries) in releases {
        let mut installed = entries
            .iter()
            .copied()
            .filter(|row| row["installed"] == true);
        // The core returns releases by version descending, including locally rebuilt releases.
        let Some(head) = installed.next() else {
            continue;
        };
        let current = value_text(head, "version");
        let mut item = UpdateItem::new("template", id, value_text(head, "name"));
        item.current_version = Some(current.to_owned());
        let retained = std::iter::once(current)
            .chain(installed.map(|row| value_text(row, "version")))
            .collect::<Vec<_>>()
            .join(", ");
        item.detail = format!("Installed releases: {retained}.");
        if let Some(bundled) = entries.iter().find(|row| {
            row["verified"] == true
                && (row["target"] == head["target"] || row["target"] == "any")
                && newer(value_text(row, "version"), current)
        }) {
            let version = value_text(bundled, "version");
            item.state = "available";
            item.available_version = Some(version.to_owned());
            item.action = Some("install_template");
            item.reference = Some(format!("{id}@{version}"));
            append_detail(
                &mut item,
                "A newer verified release is ready to install from this catalog.",
            );
            updates.push(item);
            continue;
        }

        let can_rebuild = safe_catalog_path(ctx, value_text(head, "path"))
            .is_ok_and(|path| path.join("build-latest.sh").is_file());
        if can_rebuild {
            item.action = Some("rebuild_template");
            item.reference = Some(format!("{id}@{current}"));
        }
        if let Some(feed) = starter_feed(id) {
            let project = ctx
                .home
                .join("templates")
                .join(id)
                .join(current)
                .join("project");
            let pinned = dependency_version(feed, &project);
            match (pinned, prefs.observations.get(feed.id())) {
                (Some(pinned), Some(observation)) => {
                    if let Some(error) = &observation.error {
                        item.state = "error";
                        append_detail(&mut item, &format!("{} check failed: {error}", feed.name()));
                        if let Some(version) = &observation.compatible {
                            append_detail(&mut item, &format!("Last successful recipe-range result: {version}."));
                        }
                    } else if let Some(upstream) = &observation.compatible {
                        item.state = if newer(upstream, &pinned) { "available" } else { "current" };
                        append_detail(&mut item, &format!("Bundled {} {pinned}. Latest within the recipe’s version range: {upstream}. Rebuild resolves the complete dependency set.", feed.name()));
                    } else {
                        append_detail(&mut item, &format!("Bundled {} {pinned}. The release feed has no stable version within the recipe’s range.", feed.name()));
                    }
                }
                (Some(pinned), None) => append_detail(&mut item, &format!("Bundled {} {pinned}. Check for a newer upstream release.", feed.name())),
                (None, _) => append_detail(&mut item, &format!("Could not identify the installed {} pin. Rebuild can refresh and verify dependencies.", feed.name())),
            }
        } else if id == "swift/swiftui-app" {
            item.state = "current";
            append_detail(&mut item, "This starter uses SwiftUI from your local macOS SDK. Devline can rebuild the starter; update Xcode or Command Line Tools separately.");
        } else if id == "ruby/json-cli" {
            append_detail(
                &mut item,
                "Current in the catalog. This starter uses Ruby’s standard library.",
            );
        } else {
            append_detail(
                &mut item,
                "Upstream dependency checks are unavailable for this starter.",
            );
        }
        updates.push(item);
    }
    Ok(updates)
}

fn runtime_version(feed: Feed, detail: &str) -> Option<&str> {
    let (_, output) = detail.rsplit_once(" · ")?;
    let version = match feed {
        Feed::Go => output.split_once("go version go")?.1,
        Feed::Rust => output.split_once("cargo ")?.1,
        Feed::Ruby => output.split_once("ruby ")?.1,
        Feed::Java => output.split_once("Java version: ")?.1,
        Feed::Maven => output.split_once("Apache Maven ")?.1,
        Feed::Elixir => output.rsplit_once("Elixir ")?.1,
        Feed::Erlang => output.split_once("Erlang/OTP ")?.1,
        Feed::Node => output.strip_prefix('v')?,
        Feed::Dotnet | Feed::Python => output,
        _ => return None,
    };
    number_at_start(version)
}

fn runtime_updates(ctx: &Context, prefs: &Settings) -> Result<Vec<UpdateItem>> {
    let probes = templates::runtimes(ctx)?;
    let mut updates = Vec::new();
    for row in probes.as_array().context("Read runtime toolchains")? {
        let id = value_text(row, "id");
        let feeds: &[Feed] = match id {
            "go" => &[Feed::Go],
            "python" => &[Feed::Python],
            "rust" => &[Feed::Rust],
            "ruby" => &[Feed::Ruby],
            "dotnet" => &[Feed::Dotnet],
            "java" => &[Feed::Java, Feed::Maven],
            "elixir" => &[Feed::Elixir, Feed::Erlang],
            "node" => &[Feed::Node],
            _ => &[],
        };
        for &feed in feeds {
            let mut item = UpdateItem::new("runtime", feed.id(), feed.name());
            let detail = value_text(row, "detail");
            let current = runtime_version(feed, detail);
            item.current_version = current.map(str::to_owned);
            if current.is_none() {
                item.detail = detail.to_owned();
                item.open(feed.page());
            } else if row["available"] != true {
                item.detail = detail.to_owned();
            }
            if let Some(observation) = prefs.observations.get(feed.id()) {
                if let Some(error) = &observation.error {
                    item.state = "error";
                    append_detail(&mut item, &format!("Upstream check failed: {error}"));
                    if let Some(version) = &observation.version {
                        item.available_version = current
                            .is_none_or(|current| newer(version, current))
                            .then(|| version.to_owned());
                        append_detail(
                            &mut item,
                            &format!("Last successful upstream result: {version}."),
                        );
                    }
                    item.open(feed.page());
                } else if let Some(upstream) = &observation.version {
                    let current_parts = current.and_then(stable_version);
                    let upstream_parts = stable_version(upstream).unwrap_or_default();
                    // These feeds or probes only identify a major release. Keep the
                    // comparison at that precision and report what was checked.
                    if matches!(feed, Feed::Java | Feed::Erlang)
                        && current_parts.as_ref().and_then(|v| v.first()) == upstream_parts.first()
                    {
                        item.state = "unchecked";
                        append_detail(&mut item, &format!("Matches the upstream major release ({upstream}). Patch level was not compared."));
                        item.open(feed.page());
                    } else if current.is_none_or(|current| newer(upstream, current)) {
                        item.state = "available";
                        item.available_version = Some(upstream.to_owned());
                        append_detail(&mut item, &format!("Latest upstream release: {upstream}. Use your runtime manager or the official installer."));
                        item.open(feed.page());
                    } else {
                        item.state = "current";
                        append_detail(
                            &mut item,
                            &format!("Current against the official stable release ({upstream})."),
                        );
                    }
                }
            } else {
                append_detail(&mut item, "Check for the latest official release.");
            }
            updates.push(item);
        }
    }
    Ok(updates)
}

fn app_update(prefs: &Settings) -> UpdateItem {
    let mut item = UpdateItem::new("app", "devkit", "Devline");
    let current = env!("CARGO_PKG_VERSION");
    item.current_version = Some(current.to_owned());
    let Some(feed) = &prefs.app_feed else {
        item.state = "unconfigured";
        item.detail =
            "App update checks are disabled. Add an HTTPS release manifest to enable them."
                .to_owned();
        return item;
    };
    match prefs.observations.get("app") {
        Some(observation) if observation.error.is_some() => {
            item.state = "error";
            item.detail = format!(
                "App release check failed: {}",
                observation.error.as_deref().unwrap_or_default()
            );
            if let Some(version) = observation
                .version
                .as_deref()
                .filter(|version| newer(version, current))
            {
                item.available_version = Some(version.to_owned());
                append_detail(
                    &mut item,
                    &format!("Last successful channel result: {version}."),
                );
                if let Some(url) = observation
                    .url
                    .as_deref()
                    .filter(|url| https_url(url).is_ok())
                {
                    item.open(url);
                }
            }
        }
        Some(observation) => {
            if let Some(version) = &observation.version {
                if newer(version, current) {
                    item.state = "available";
                    item.available_version = Some(version.clone());
                    item.detail = format!(
                        "Version {version} is available from the configured release channel."
                    );
                    if let Some(url) = observation
                        .url
                        .as_deref()
                        .filter(|url| https_url(url).is_ok())
                    {
                        item.open(url);
                    }
                } else {
                    item.state = "current";
                    item.detail =
                        format!("Current against the configured release channel ({version}).");
                }
            }
        }
        None => {
            item.detail = format!(
                "Release channel configured at {feed}. Run a check to read its latest version."
            )
        }
    }
    item
}

fn make_status(ctx: &Context, prefs: Settings) -> Result<Value> {
    let mut items = vec![app_update(&prefs)];
    items.extend(docs_updates(ctx, &prefs)?);
    items.extend(template_updates(ctx, &prefs)?);
    items.extend(runtime_updates(ctx, &prefs)?);
    Ok(json!({"checked_at": prefs.checked_at, "items": items}))
}

pub fn status(ctx: &Context) -> Result<Value> {
    make_status(ctx, settings(ctx)?)
}

pub(crate) fn app_status(ctx: &Context) -> Result<Value> {
    let prefs = settings(ctx)?;
    let checked_at = prefs
        .observations
        .contains_key("app")
        .then_some(prefs.app_checked_at.or(prefs.checked_at))
        .flatten();
    Ok(json!({"checked_at":checked_at,"items":[app_update(&prefs)]}))
}

pub(crate) fn app_check(ctx: &Context, request: &Value) -> Result<Value> {
    if request["offline"] == true {
        bail!("Checking app releases requires a connection. Saved app status works offline.");
    }
    let before = settings(ctx)?;
    let Some(url) = before.app_feed.as_deref() else {
        return app_status(ctx);
    };
    let started = now().as_nanos() as u64;
    let mut observation = observe_app(url);
    let _lock = lock_library(ctx)?;
    let mut prefs = settings(ctx)?;
    if prefs.app_feed == before.app_feed
        && prefs.app_check_started_at.is_none_or(|old| started >= old)
    {
        if observation.error.is_some() {
            if let Some(previous) = prefs.observations.get("app") {
                observation.version.clone_from(&previous.version);
                observation.url.clone_from(&previous.url);
            }
        }
        prefs.observations.insert("app".to_owned(), observation);
        prefs.app_checked_at = Some(now().as_secs());
        prefs.app_check_started_at = Some(started);
        save(ctx, &prefs)?;
    }
    drop(_lock);
    app_status(ctx)
}

pub fn check(ctx: &Context) -> Result<Value> {
    let before = settings(ctx)?;
    let check_started = now().as_nanos() as u64;
    let sources: BTreeSet<&str> =
        ctx.catalog
            .docs
            .iter()
            .map(|pack| pack.id.as_str())
            .chain(
                ctx.catalog
                    .templates
                    .iter()
                    .map(|starter| starter.stack.as_str()),
            )
            .chain(
                ctx.catalog
                    .templates
                    .iter()
                    .filter_map(|starter| starter_feed(&starter.id).map(Feed::id)),
            )
            .chain(ctx.catalog.templates.iter().filter_map(
                |starter| match starter.stack.as_str() {
                    "elixir" => Some("erlang"),
                    "java" => Some("maven"),
                    _ => None,
                },
            ))
            .collect();
    let mut observations = std::thread::scope(|scope| {
        let mut jobs = Vec::new();
        for feed in Feed::ALL
            .into_iter()
            .filter(|feed| sources.contains(feed.id()))
        {
            jobs.push((feed.id(), scope.spawn(move || observe(feed))));
        }
        if let Some(url) = &before.app_feed {
            jobs.push(("app", scope.spawn(move || observe_app(url))));
        }
        jobs.into_iter()
            .map(|(id, job)| {
                let observation = job
                    .join()
                    .map_err(|_| anyhow::anyhow!("Update check worker failed"))?;
                Ok((id.to_owned(), observation))
            })
            .collect::<Result<BTreeMap<_, _>>>()
    })?;

    let _lock = lock_library(ctx)?;
    let mut prefs = settings(ctx)?;
    if prefs.app_feed != before.app_feed {
        observations.remove("app");
        if let Some(app) = prefs.observations.get("app") {
            observations.insert("app".to_owned(), app.clone());
        }
    }
    if prefs
        .app_check_started_at
        .is_some_and(|previous| previous > check_started)
    {
        observations.remove("app");
        if let Some(app) = prefs.observations.get("app") {
            observations.insert("app".to_owned(), app.clone());
        }
    } else if prefs.app_feed == before.app_feed && observations.contains_key("app") {
        prefs.app_checked_at = Some(now().as_secs());
        prefs.app_check_started_at = Some(check_started);
    }
    if prefs
        .check_started_at
        .is_none_or(|previous| check_started >= previous)
    {
        for (id, observation) in &mut observations {
            if observation.error.is_some() {
                if let Some(previous) = prefs.observations.get(id) {
                    observation.version.clone_from(&previous.version);
                    observation.compatible.clone_from(&previous.compatible);
                    observation.url.clone_from(&previous.url);
                }
            }
        }
        prefs.check_started_at = Some(check_started);
        prefs.checked_at = Some(now().as_secs());
        prefs.observations = observations;
        save(ctx, &prefs)?;
    }
    drop(_lock);
    make_status(ctx, prefs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_and_npm_updates_use_stable_publisher_versions() {
        let versions = feed_versions(
            Feed::Node,
            r#"[{"version":"v26.13.0"},{"version":"v27.0.0-rc.1"},{"version":"v26.12.1"}]"#,
        )
        .unwrap();
        assert_eq!(latest(versions.iter().map(String::as_str)), Some("26.13.0"));
        assert_eq!(
            runtime_version(Feed::Node, "/tools/node · v26.13.0"),
            Some("26.13.0")
        );
        for feed in [
            Feed::React,
            Feed::Vite,
            Feed::TypeScript,
            Feed::Playwright,
            Feed::Fastify,
        ] {
            assert_eq!(
                feed_versions(feed, r#"{"version":"1.2.3"}"#).unwrap(),
                ["1.2.3"]
            );
            assert!(feed_versions(feed, r#"{"version":"2.0.0-beta.1"}"#).is_err());
        }
    }

    #[test]
    fn compares_numeric_stable_versions() {
        assert!(newer("3.15.10", "3.15.9"));
        assert!(!newer("3.15.0", "3.15"));
        assert!(!newer("4.2.0-rc.1", "4.1.9"));
        assert_eq!(
            latest(["4.1.9", "4.1.10", "4.2.0-M1"].into_iter()),
            Some("4.1.10")
        );
    }

    #[test]
    fn validates_app_urls_and_schema() {
        for url in [
            "http://example.org/update.json",
            "https://x:y@example.org/feed",
            "https://example.org/feed?q=token",
            "https://example.org/feed#section",
            "https://example.org:0/feed",
            "https://example.org\\evil",
        ] {
            assert!(https_url(url).is_err(), "{url}");
        }
        assert!(https_url("https://localhost:8443/releases/latest.json").is_ok());
        let manifest = parse_app_manifest(
            r#"{"schema_version":1,"version":"1.2.3","url":"https://example.org/releases"}"#,
        )
        .unwrap();
        assert_eq!(manifest.version, "1.2.3");
        assert!(parse_app_manifest(
            r#"{"schema_version":2,"version":"1.2.3","url":"https://example.org/releases"}"#
        )
        .is_err());
    }

    #[test]
    fn reads_upstream_feeds_and_ignores_prerelease_versions() {
        let tauri = feed_versions(Feed::Tauri, r#"{"versions":[{"num":"2.12.5","yanked":false},{"num":"2.12.6","yanked":true},{"num":"3.0.0-alpha.1","yanked":false},{"num":"3.0.0","yanked":false}]}"#).unwrap();
        let compatible = tauri
            .iter()
            .map(String::as_str)
            .filter(|v| stable_version(v).is_some_and(|parts| Feed::Tauri.compatible(&parts)));
        assert_eq!(latest(compatible), Some("2.12.5"));
        let python = feed_versions(Feed::Python, r#"<a href="/downloads/release/python-3152/">Latest Python 3 Release - Python 3.15.2</a>"#).unwrap();
        assert_eq!(python, ["3.15.2"]);
        let ruby = feed_versions(
            Feed::Ruby,
            "The current stable version is <strong>4.0.9</strong>.",
        )
        .unwrap();
        assert_eq!(ruby, ["4.0.9"]);
        let rust = feed_versions(Feed::Rust, "[pkg.rust]\nversion = \"1.99.1 (abc)\"\n").unwrap();
        assert_eq!(rust, ["1.99.1"]);
        let go = feed_versions(
            Feed::Go,
            r#"[{"version":"go1.27.3","stable":true},{"version":"go1.28rc1","stable":false}]"#,
        )
        .unwrap();
        assert_eq!(go, ["1.27.3"]);
        let xml = "<version>4.1.10</version><version>4.2.0</version><version>4.1.11-M1</version>";
        let spring = feed_versions(Feed::Spring, xml).unwrap();
        let compatible = spring
            .iter()
            .map(String::as_str)
            .filter(|v| stable_version(v).is_some_and(|parts| Feed::Spring.compatible(&parts)));
        assert_eq!(latest(compatible), Some("4.1.10"));
    }
}
