mod manifests;
mod version;

use crate::{
    doc_updates,
    updates::{compare_doc_releases, doc_release},
    Context,
};
use anyhow::{bail, Result};
use manifests::{detect, Dependency, Detected};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use version::{matches, Rule};

#[derive(Serialize)]
struct Pin {
    id: String,
    version: String,
}

#[derive(Serialize)]
struct Docset {
    id: String,
    installed_versions: Vec<String>,
    available_versions: Vec<String>,
    prepare_version: Option<String>,
    version: Option<String>,
    state: &'static str,
    compatible: Option<bool>,
    requirement: Option<String>,
    detail: String,
}

struct Selection {
    pins: Vec<Pin>,
    docsets: Vec<Docset>,
}

#[derive(Serialize)]
pub(crate) struct ProjectOrigin {
    pub starter: String,
    pub version: String,
    pub reference: String,
    pub digest: String,
}

pub(crate) fn origin(path: &Path) -> Option<ProjectOrigin> {
    const LIMIT: u64 = 4096;
    let marker = path.join(if crate::project_layout::is_organized(path) {
        crate::project_layout::ORIGIN
    } else {
        crate::project_layout::LEGACY_ORIGIN
    });
    let metadata = fs::symlink_metadata(&marker).ok()?;
    if !metadata.is_file() || metadata.len() > LIMIT {
        return None;
    }
    let mut bytes = Vec::new();
    fs::File::open(marker)
        .ok()?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > LIMIT {
        return None;
    }
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    let starter = value["template"].as_str()?;
    crate::templates::id_parts(starter).ok()?;
    let version = value["version"].as_str()?;
    if version.len() > 64
        || version.split('.').count() != 3
        || !version.split('.').all(|part| part.parse::<u64>().is_ok())
    {
        return None;
    }
    let digest = value["digest"].as_str()?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(ProjectOrigin {
        starter: starter.into(),
        version: version.into(),
        reference: format!("{starter}@{version}"),
        digest: digest.into(),
    })
}

fn ensure(db: &Connection) -> Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS project_doc_pins (
             path TEXT NOT NULL,
             pack_id TEXT NOT NULL,
             version TEXT NOT NULL,
             PRIMARY KEY(path,pack_id)
         );",
    )?;
    Ok(())
}

fn pins(db: &Connection, path: &Path) -> Result<Vec<Pin>> {
    ensure(db)?;
    let mut statement = db.prepare_cached(
        "SELECT pack_id,version FROM project_doc_pins WHERE path=?1 ORDER BY pack_id",
    )?;
    let rows = statement.query_map([path.to_string_lossy().as_ref()], |row| {
        Ok(Pin {
            id: row.get(0)?,
            version: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

fn version_order(versions: &mut Vec<String>) {
    versions.sort_by(|a, b| compare_doc_releases(b, a).then_with(|| b.cmp(a)));
    versions.dedup();
}

fn selection(ctx: &Context, db: &Connection, detected: &Detected) -> Result<Selection> {
    let pins = pins(db, &detected.path)?;
    let ids: BTreeSet<&str> = detected
        .dependencies
        .iter()
        .map(|item| item.id)
        .chain(pins.iter().map(|pin| pin.id.as_str()))
        .collect();
    if ids.is_empty() {
        return Ok(Selection {
            pins,
            docsets: Vec::new(),
        });
    }
    let preferred = doc_updates::preferences(db)?;
    let remote = doc_updates::installed_metadata(db)?;
    let mut available = ctx.catalog.docs.clone();
    available.extend(
        doc_updates::cached_packs(ctx)?
            .into_iter()
            .map(|(pack, _)| pack),
    );
    available.extend(doc_updates::checked_packs(ctx)?);
    let mut statement = db.prepare_cached("SELECT id,version FROM packs")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut installed = BTreeMap::<String, Vec<String>>::new();
    for row in rows {
        let (id, release) = row?;
        installed.entry(id).or_default().push(release);
    }
    let mut metadata = BTreeMap::new();
    for pack in available.iter().chain(remote.values()) {
        if let Some(upstream) = &pack.upstream_version {
            metadata.insert((pack.id.as_str(), pack.version.as_str()), upstream.as_str());
        }
    }
    let compatible = |dependency: &Dependency, release: &str| {
        let source_version = metadata
            .get(&(dependency.id, release))
            .copied()
            .or_else(|| {
                // These built-in snapshots predate metadata and were ingested with the .NET 10 moniker.
                (matches!(dependency.id, "csharp" | "aspnet")
                    && doc_release(release).0 == "2026.11.06")
                    .then_some("10.0")
            })
            .unwrap_or(release);
        if matches!(dependency.id, "csharp" | "aspnet")
            && doc_release(source_version).0.split('.').count() > 2
        {
            // A dated snapshot has no meaningful numeric relationship to .NET.
            let first = source_version
                .split('.')
                .next()
                .and_then(|value| value.parse::<u32>().ok());
            if first.is_some_and(|value| value >= 1000) {
                return None;
            }
        }
        matches(&dependency.rule, source_version)
    };
    let mut docsets = Vec::with_capacity(ids.len());
    for id in ids {
        let mut installed_versions = installed.remove(id).unwrap_or_default();
        version_order(&mut installed_versions);
        let mut available_versions = available
            .iter()
            .filter(|pack| pack.id == id)
            .map(|pack| pack.version.clone())
            .collect::<Vec<_>>();
        version_order(&mut available_versions);
        let dependency = detected.dependencies.iter().find(|item| item.id == id);
        let requirement =
            dependency.and_then(|item| item.version.clone().or_else(|| item.requirement.clone()));
        let mut docset = Docset {
            id: id.to_owned(),
            installed_versions,
            available_versions,
            prepare_version: None,
            version: None,
            state: "uninstalled",
            compatible: None,
            requirement,
            detail: String::new(),
        };
        if let Some(pin) = pins.iter().find(|pin| pin.id == id) {
            docset.state = "pinned";
            docset.version = Some(pin.version.clone());
            docset.compatible =
                dependency.and_then(|dependency| compatible(dependency, &pin.version));
            let missing = !docset.installed_versions.contains(&pin.version);
            docset.detail = if docset.compatible == Some(false) {
                let state = if missing { " is missing and" } else { "" };
                format!(
                    "Pinned {}{state} differs from the detected {} requirement ({}).",
                    pin.version,
                    id,
                    docset.requirement.as_deref().unwrap_or_default()
                )
            } else if missing {
                format!("Pinned {} is not installed.", pin.version)
            } else {
                format!("Pinned to {} for this project.", pin.version)
            };
        } else if let Some(dependency) = dependency {
            choose(
                &mut docset,
                dependency,
                preferred.get(id).map(String::as_str),
                &compatible,
            );
        }
        docset.prepare_version = docset.version.clone().or_else(|| {
            let dependency = dependency?;
            docset
                .available_versions
                .iter()
                .find(|release| {
                    matches!(dependency.rule, Rule::Any)
                        || compatible(dependency, release) == Some(true)
                })
                .cloned()
        });
        docsets.push(docset);
    }
    Ok(Selection { pins, docsets })
}

fn choose(
    docset: &mut Docset,
    dependency: &Dependency,
    preferred: Option<&str>,
    compatible: &impl Fn(&Dependency, &str) -> Option<bool>,
) {
    let label = docset
        .requirement
        .as_deref()
        .unwrap_or("an unspecified version");
    if docset.installed_versions.is_empty() {
        if docset.id == "swiftui" && docset.available_versions.is_empty() {
            docset.detail = "Prepare Apple’s SwiftUI API reference from the selected local Xcode SDK in Library → From This Mac, or run `devline --offline docs install swiftui`. Website tutorials are separate from the SDK reference.".into();
            return;
        }
        if let Some(release) = docset
            .available_versions
            .iter()
            .find(|release| compatible(dependency, release) == Some(true))
        {
            docset.detail = format!(
                "Install {release} to match {label} detected in {}.",
                dependency.source
            );
        } else if docset
            .available_versions
            .iter()
            .any(|release| compatible(dependency, release) == Some(false))
        {
            docset.detail = format!(
                "No installed release. The bundled versions do not match {label} from {}.",
                dependency.source
            );
        } else {
            docset.detail = format!(
                "No documentation for {} is installed. Detected {label} in {}.",
                docset.id, dependency.source
            );
        }
        return;
    }
    let unversioned = matches!(dependency.rule, Rule::Any);
    let usable = |release| unversioned || compatible(dependency, release) == Some(true);
    let chosen = preferred
        .filter(|release| {
            docset
                .installed_versions
                .iter()
                .any(|installed| installed == release)
                && usable(release)
        })
        .or_else(|| {
            docset
                .installed_versions
                .iter()
                .map(String::as_str)
                .find(|release| usable(release))
        });
    if let Some(chosen) = chosen {
        docset.version = Some(chosen.to_owned());
        if unversioned {
            docset.state = "unversioned";
            docset.detail = format!(
                "Using {chosen}. No project version is declared for {}.",
                docset.id
            );
        } else {
            docset.state = "matched";
            docset.compatible = Some(true);
            let expected = if matches!(dependency.rule, Rule::Series(..)) {
                "the version line "
            } else {
                ""
            };
            docset.detail = format!(
                "{chosen} matches {expected}{label} from {}.",
                dependency.source
            );
        }
    } else {
        docset.state = "unmatched";
        let tested = docset
            .installed_versions
            .iter()
            .any(|release| compatible(dependency, release).is_some());
        docset.compatible = tested.then_some(false);
        docset.detail = if tested {
            format!(
                "Installed releases do not match {label} from {}.",
                dependency.source
            )
        } else {
            format!("Could not match {label} from {} to an installed release. Choose an explicit project pin to proceed.", dependency.source)
        };
    }
}

fn context_value(ctx: &Context, db: &Connection, detected: &Detected) -> Result<Value> {
    let scope = selection(ctx, db, detected)?;
    Ok(json!({
        "path": detected.path,
        "stacks": detected.stacks,
        "stack": detected.stack(),
        "dependencies": detected.dependencies,
        "pins": scope.pins,
        "docsets": scope.docsets,
        "origin": origin(&detected.path),
    }))
}

pub(super) fn detected_context(path: &Path) -> Value {
    detect(path).map_or_else(
        |_| json!({"path":path,"stacks":[],"stack":null,"dependencies":[]}),
        |detected| json!({"path":detected.path,"stacks":detected.stacks,"stack":detected.stack(),"dependencies":detected.dependencies,"origin":origin(&detected.path)}),
    )
}

pub(super) fn context(ctx: &Context, path: &Path) -> Result<Value> {
    let detected = detect_with_library(ctx, path)?;
    context_value(ctx, &ctx.database()?, &detected)
}

pub(super) fn root(ctx: &Context, path: &Path) -> Result<PathBuf> {
    manifests::root(path, |root| installed_origin_stack(ctx, root))
}

fn detect_with_library(ctx: &Context, path: &Path) -> Result<Detected> {
    manifests::detect_with_origin(path, |root| installed_origin_stack(ctx, root))
}

fn installed_origin_stack(ctx: &Context, root: &Path) -> Option<String> {
    let origin = origin(root)?;
    let (stack, _) = crate::templates::id_parts(&origin.starter).ok()?;
    if crate::project_prepare::has_generated_receipt(ctx, root, stack) {
        return Some(stack.to_owned());
    }
    let installed = crate::templates::installed_base(ctx, &origin.starter, false)
        .ok()?
        .join(&origin.version);
    let release = crate::templates::verify_release(&installed).ok()?;
    (release.template.id == origin.starter
        && release.template.version == origin.version
        && release.template.stack == stack
        && release.digest == origin.digest)
        .then_some(stack.to_owned())
}

fn valid_selector(input: &str) -> bool {
    !input.is_empty()
        && input.len() <= 128
        && input
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._+".contains(&byte))
}

pub(super) fn pin(ctx: &Context, path: &Path, id: &str, version: &str) -> Result<Value> {
    if !valid_selector(id) || !valid_selector(version) {
        bail!("Specify a documentation pack and exact release, such as fastapi@0.162.0.");
    }
    let detected = detect_with_library(ctx, path)?;
    let mut db = ctx.database()?;
    ensure(&db)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let known = ctx.catalog.docs.iter().any(|pack| pack.id == id)
        || transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM packs WHERE id=?1)",
            [id],
            |row| row.get(0),
        )?;
    if !known {
        bail!("Unknown documentation pack {id}.");
    }
    transaction.execute(
        "INSERT INTO project_doc_pins(path,pack_id,version) VALUES(?1,?2,?3)
         ON CONFLICT(path,pack_id) DO UPDATE SET version=excluded.version",
        params![detected.path.to_string_lossy(), id, version],
    )?;
    let result = context_value(ctx, &transaction, &detected)?;
    transaction.commit()?;
    Ok(result)
}

pub(super) fn unpin(ctx: &Context, path: &Path, id: &str) -> Result<Value> {
    let detected = detect_with_library(ctx, path)?;
    let mut db = ctx.database()?;
    ensure(&db)?;
    let transaction = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    transaction.execute(
        "DELETE FROM project_doc_pins WHERE path=?1 AND pack_id=?2",
        params![detected.path.to_string_lossy(), id],
    )?;
    let result = context_value(ctx, &transaction, &detected)?;
    transaction.commit()?;
    Ok(result)
}

pub(super) fn search_releases(
    ctx: &Context,
    db: &Connection,
    path: &Path,
) -> Result<Option<BTreeMap<String, String>>> {
    let detected = detect_with_library(ctx, path)?;
    let scope = selection(ctx, db, &detected)?;
    if scope.docsets.is_empty() {
        return Ok(None);
    }
    let releases = scope
        .docsets
        .into_iter()
        .filter_map(|pack| {
            let version = pack.version?;
            pack.installed_versions
                .contains(&version)
                .then_some((pack.id, version))
        })
        .collect();
    Ok(Some(releases))
}

pub(super) fn runtime_compatibility(
    path: &Path,
    stack: &str,
    installed_version: &str,
) -> Result<Option<bool>> {
    let detected = detect(path)?;
    let id = if stack == "dotnet" { "csharp" } else { stack };
    Ok(detected
        .dependencies
        .iter()
        .find(|dependency| dependency.id == id)
        .and_then(|dependency| matches(&dependency.rule, installed_version)))
}
