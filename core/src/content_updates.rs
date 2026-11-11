//! One view of the independent update channels. Only existing library content is
//! eligible for bulk updates; runtime and app rows remain informational actions.
use crate::{apple_docs, doc_updates, infrastructure, tasks::Job, templates, updates, Context};
use anyhow::{bail, Context as _, Result};
use fs2::FileExt;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    sync::atomic::{AtomicU64, Ordering},
};

static JOURNAL_ID: AtomicU64 = AtomicU64::new(0);
const PENDING_FILE: &str = "content-update-pending.json";

fn pending(ctx: &Context) -> Result<Vec<Value>> {
    let path = ctx.home.join(PENDING_FILE);
    let file = match OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context("Read unfinished content updates"),
    };
    if !file.metadata()?.is_file() {
        bail!("Invalid unfinished content update journal");
    }
    let mut bytes = Vec::new();
    file.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 2 * 1024 * 1024 {
        bail!("Unfinished content update journal is too large");
    }
    let saved: Value =
        serde_json::from_slice(&bytes).context("Read unfinished content update journal")?;
    let items = saved["items"]
        .as_array()
        .context("Invalid unfinished content updates")?;
    if saved["schema_version"] != 1
        || items.len() > 1000
        || items.iter().any(|item| {
            !is_content_update(item) || text(item, "id").is_empty() || text(item, "name").is_empty()
        })
    {
        bail!("Invalid unfinished content update journal");
    }
    Ok(items.clone())
}

fn save_pending(ctx: &Context, items: &[Value]) -> Result<()> {
    let path = ctx.home.join(format!(
        ".content-updates-{}-{}.tmp",
        std::process::id(),
        JOURNAL_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)?;
        serde_json::to_writer(&mut file, &json!({"schema_version":1,"items":items}))?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&path, ctx.home.join(PENDING_FILE))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result.context("Save unfinished content updates")
}

fn with_pending(ctx: &Context, mut snapshot: Value) -> Result<Value> {
    let items = snapshot["items"]
        .as_array_mut()
        .context("Invalid content snapshot")?;
    for mut item in pending(ctx)? {
        item["pending"] = json!(true);
        item["detail"] = json!(format!(
            "This exact update still needs preparation. {}",
            item["retry_detail"]
                .as_str()
                .unwrap_or("Retry to complete its offline verification.")
        ));
        if let Some(current) = items.iter_mut().find(|current| current["id"] == item["id"]) {
            *current = item;
        } else {
            items.push(item);
        }
    }
    snapshot["available_content"] =
        json!(items.iter().filter(|item| is_content_update(item)).count());
    Ok(snapshot)
}

fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map_or(&[], Vec::as_slice)
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_default()
}

fn source(id: &str, name: &str, value: &Value, error: Option<&str>) -> Value {
    let failures = rows(value, "items")
        .iter()
        .filter(|item| item["state"] == "error" || item["state"] == "republished")
        .count() as u64
        + value["additional_publisher_failures"].as_u64().unwrap_or(0);
    let error = error.or_else(|| value["error"].as_str());
    let state = if error.is_some() || failures > 0 {
        "error"
    } else if value["checked_at"].is_null() {
        "unchecked"
    } else {
        "current"
    };
    json!({"id":id,"name":name,"checked_at":value["checked_at"],"state":state,
        "detail":error.map(str::to_owned).unwrap_or_else(|| if failures > 0 {
            format!("{failures} publisher checks failed. Saved results remain available.")
        } else if state == "unchecked" { "Not checked online yet.".to_owned() }
        else { "Saved publisher results are available offline.".to_owned() })})
}

fn compose(
    upstream: Value,
    docs: Value,
    images: Value,
    vault: Value,
    local: Value,
    errors: &[(&str, String)],
) -> Value {
    let error = |id: &str| {
        errors
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value.as_str())
    };
    let mut sources = vec![
        source(
            "docs",
            "Official documentation releases",
            &docs,
            error("docs"),
        ),
        source(
            "upstream",
            "Starter dependencies, runtimes, and app",
            &upstream,
            error("upstream"),
        ),
        source("images", "Image publishers", &images, error("images")),
    ];
    if let Some(detail) = error("vault") {
        sources.push(json!({"id":"vault","name":"Local image library","checked_at":null,"state":"error","detail":detail}));
    }
    let mut items: Vec<_> = rows(&upstream, "items")
        .iter()
        .filter(|item| item["kind"] != "docs")
        .cloned()
        .collect();
    if error("upstream").is_some() {
        for item in &mut items {
            item["state"] = json!("error");
            item["action"] = Value::Null;
        }
    }
    let docs_failed = error("docs").is_some() || docs["error"].is_string();
    for pack in rows(&docs, "packs") {
        // Install new docset families explicitly from the Library.
        if rows(pack, "installed_versions").is_empty() {
            continue;
        }
        let fallback = rows(&upstream, "items")
            .iter()
            .find(|item| item["id"] == format!("docs:{}", text(pack, "id")));
        let latest = &pack["latest"];
        if let Some(fallback) = fallback {
            let prefer_local = fallback["available_version"]
                .as_str()
                .filter(|_| fallback["action"] == "install_docs")
                .is_some_and(|local_version| {
                    docs_failed
                        || latest["pack"]["version"].as_str().is_none_or(|remote| {
                            !updates::compare_doc_releases(local_version, remote).is_lt()
                        })
                });
            if prefer_local || (!docs_failed && latest.is_null()) {
                let mut item = fallback.clone();
                if item["action"] == "install_docs" {
                    item["delivery"] = json!("local");
                }
                items.push(item);
                continue;
            }
        }
        let version = latest["pack"]["version"].as_str();
        let available = !docs_failed && pack["state"] == "available" && version.is_some();
        let state = if docs_failed {
            "error"
        } else if available {
            "available"
        } else if pack["state"] == "current" {
            "current"
        } else {
            "unchecked"
        };
        items.push(json!({"id":format!("docs:{}", text(pack,"id")),"kind":"docs","name":pack["name"],
            "current_version":pack["active_version"],"available_version":version,"state":state,
            "detail": if docs_failed { "The documentation channel check failed. Retry before applying saved offers." }
                else if available { "Install the official release. Project documentation pins keep their selected versions." }
                else if state == "current" { "Your active reference matches the checked documentation channel." }
                else { "Check the official documentation channel for downloadable releases." },
            "action":available.then_some("install_docs"),
            "reference":version.filter(|_|available).map(|version|format!("{}@{version}",text(pack,"id"))),"url":null}));
    }
    // Bundled releases remain usable even if the remote channel is disabled or unreadable.
    for fallback in rows(&upstream, "items")
        .iter()
        .filter(|item| item["kind"] == "docs" && item["id"] != "docs:swiftui")
    {
        if !items.iter().any(|item| item["id"] == fallback["id"]) {
            let mut item = fallback.clone();
            if item["action"] == "install_docs" {
                item["delivery"] = json!("local");
            } else if docs_failed {
                item["state"] = json!("error");
                item["action"] = Value::Null;
            }
            items.push(item);
        }
    }
    let archived: BTreeSet<_> = rows(&vault, "images")
        .iter()
        .filter(|row| row["archived"] == true)
        .filter_map(|row| row["id"].as_str())
        .collect();
    for item in rows(&images, "items")
        .iter()
        .filter(|item| archived.contains(text(item, "id")))
    {
        let available = error("images").is_none()
            && item["state"] == "available"
            && item["reference"].is_string();
        let state = if error("images").is_some()
            || item["state"] == "error"
            || item["state"] == "republished"
        {
            "error"
        } else if available {
            "available"
        } else if item["state"] == "up_to_date" {
            "current"
        } else {
            "unchecked"
        };
        items.push(json!({"id":format!("image:{}",text(item,"id")),"kind":"image","name":item["name"],
            "current_version":item["current_version"],"available_version":item["available_version"],"state":state,
            "detail":item["detail"],"action":available.then_some("update_image"),
            "reference": if available { item["reference"].clone() } else { Value::Null },"url":null}));
    }
    // Keep an already prepared Apple reference aligned with the selected local
    // SDK, without acquiring or redistributing Apple toolchains/documentation.
    if local["active_version"].is_string() {
        let available = local["available"] == true && local["installed"] != true;
        items.push(json!({"id":"docs:swiftui","kind":"docs","name":local["name"],
            "current_version":local["active_version"],"available_version":local["version"],
            "state":if available {"available"} else if local["available"] == true {"current"} else {"error"},
            "detail":local["detail"],"action":available.then_some("prepare_local_docs"),
            "reference":available.then_some("swiftui"),"url":null}));
    }
    if let Some(detail) = error("local").or_else(|| {
        (local["active_version"].is_string() && local["available"] != true)
            .then(|| local["detail"].as_str())
            .flatten()
    }) {
        sources.push(json!({"id":"local","name":"Local SDK reference","checked_at":null,"state":"error","detail":detail}));
    }
    let available_content = items.iter().filter(|item| is_content_update(item)).count();
    let all_checked = sources.iter().all(|source| source["state"] == "current");
    let checked_at = if all_checked {
        sources
            .iter()
            .filter_map(|source| source["checked_at"].as_u64())
            .min()
    } else {
        None
    };
    json!({"checked_at":checked_at,"all_checked":all_checked,"items":items,"sources":sources,"available_content":available_content})
}

fn is_content_update(item: &Value) -> bool {
    item["state"] == "available"
        && matches!(
            text(item, "action"),
            "install_docs"
                | "prepare_local_docs"
                | "install_template"
                | "rebuild_template"
                | "update_image"
        )
        && item["reference"].is_string()
}

fn collect(
    value: Result<Value>,
    id: &'static str,
    errors: &mut Vec<(&'static str, String)>,
) -> Value {
    match value {
        Ok(value) => value,
        Err(error) => {
            errors.push((id, format!("{error:#}")));
            json!({})
        }
    }
}

fn stage(job: &mut Option<&mut Job>, label: &str) -> Result<()> {
    if let Some(job) = job {
        job.stage(label)?;
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    All,
    Assets,
    AssetsStatus,
}

impl Scope {
    fn updates(request: &Value) -> Self {
        if request["category"] == "assets" {
            Self::Assets
        } else {
            Self::All
        }
    }
}

pub(crate) fn status(ctx: &Context, request: &Value) -> Result<Value> {
    let scope = if request["category"] == "assets" {
        Scope::AssetsStatus
    } else {
        Scope::All
    };
    gather(ctx, false, scope, None)
}

pub(crate) fn check(ctx: &Context, request: &Value, job: Option<&mut Job>) -> Result<Value> {
    if request["offline"] == true {
        bail!(
            "Checking update publishers requires a connection. Saved update status works offline."
        );
    }
    gather(ctx, true, Scope::updates(request), job)
}

fn gather(ctx: &Context, online: bool, scope: Scope, mut job: Option<&mut Job>) -> Result<Value> {
    let assets_only = scope != Scope::All;
    let mut errors = Vec::new();
    stage(
        &mut job,
        if online {
            "Checking starter, runtime, and app publishers"
        } else {
            "Reading saved update status"
        },
    )?;
    let upstream = collect(
        if online {
            updates::check(ctx)
        } else if scope == Scope::AssetsStatus {
            updates::asset_status(ctx)
        } else {
            updates::status(ctx)
        },
        "upstream",
        &mut errors,
    );
    let docs = if assets_only {
        Value::Null
    } else {
        stage(&mut job, "Checking official documentation releases")?;
        collect(
            if online {
                doc_updates::check(ctx)
            } else {
                doc_updates::status(ctx)
            },
            "docs",
            &mut errors,
        )
    };
    stage(&mut job, "Checking image publishers")?;
    let images = collect(
        infrastructure::execute(
            ctx,
            &json!({"command":if online {"infra.updates.check"} else {"infra.updates"}}),
            None,
        ),
        "images",
        &mut errors,
    );
    stage(&mut job, "Reading local library")?;
    let vault = collect(
        infrastructure::execute(ctx, &json!({"command":"infra.status"}), None),
        "vault",
        &mut errors,
    );
    let local = if assets_only {
        Value::Null
    } else {
        collect(apple_docs::status(ctx), "local", &mut errors)
    };
    let mut snapshot = with_pending(ctx, compose(upstream, docs, images, vault, local, &errors))?;
    if assets_only {
        let items = snapshot["items"]
            .as_array_mut()
            .context("Invalid update items")?;
        items.retain(|item| item["kind"] != "docs");
        snapshot["available_content"] =
            json!(items.iter().filter(|item| is_content_update(item)).count());
        let sources = snapshot["sources"]
            .as_array_mut()
            .context("Invalid update sources")?;
        sources.retain(|source| source["id"] != "docs" && source["id"] != "local");
        let all_checked = sources.iter().all(|source| source["state"] == "current");
        snapshot["checked_at"] = if all_checked {
            json!(sources
                .iter()
                .filter_map(|source| source["checked_at"].as_u64())
                .min())
        } else {
            Value::Null
        };
        snapshot["all_checked"] = json!(all_checked);
    }
    Ok(snapshot)
}

pub(crate) fn apply(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW)
        .open(ctx.home.join("content-update.lock"))?;
    loop {
        match FileExt::try_lock_exclusive(&lock) {
            Ok(()) => break,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                stage(&mut job, "Waiting for another content update")?;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => return Err(error.into()),
        }
    }
    let scope = Scope::updates(request);
    let before = gather(ctx, false, scope, None)?;
    let selected = request["id"].as_str();
    let plan: Vec<_> = rows(&before, "items")
        .iter()
        .filter(|item| {
            is_content_update(item)
                && selected
                    .is_none_or(|selected| item["id"] == selected || item["reference"] == selected)
        })
        .cloned()
        .collect();
    if selected.is_some() && plan.is_empty() {
        bail!(
            "That update is no longer available. Refresh update status and select a current item."
        );
    }
    let mut remaining = pending(ctx)?;
    for item in &plan {
        if !remaining
            .iter()
            .any(|old| old["id"] == item["id"] && old["reference"] == item["reference"])
        {
            remaining.push(item.clone());
        }
    }
    // Record the exact plan before preparation. A failed build, cancellation,
    // or terminated process must not hide an installed-but-unverified release.
    save_pending(ctx, &remaining)?;
    let offline = request["offline"] == true;
    // An explicit install of confirmed offers reports only preparation
    // outcomes. Freshness and publisher failures remain in the snapshot.
    let include_source_issues = selected.is_none() && request["confirmed_only"] != true;
    let mut steps: Vec<Value> = rows(&before, "sources").iter()
        .filter(|source| include_source_issues && source["state"] != "current")
        .map(|source| json!({"id":format!("source:{}",text(source,"id")),"name":source["name"],
            "action":"check_source","state":"blocked","detail":format!("{} Run updates check to refresh this source.",text(source,"detail"))}))
        .collect();
    for (index, item) in plan.iter().enumerate() {
        stage(
            &mut job,
            &format!(
                "Updating {} ({} of {})",
                text(item, "name"),
                index + 1,
                plan.len()
            ),
        )?;
        let action = text(item, "action");
        let blocked = offline && action == "rebuild_template";
        let result = if blocked {
            Err(anyhow::anyhow!(
                "This update needs a connection. Existing offline releases were kept."
            ))
        } else {
            apply_item(ctx, item, offline, &mut job)
        };
        // Do not swallow cancellation or timeout and continue mutating the library.
        if let Some(job) = job.as_deref_mut() {
            job.checkpoint()?;
        }
        let (state, detail) = match result {
            Ok(_) => ("succeeded", "Update completed.".to_owned()),
            Err(error) => (
                if blocked { "blocked" } else { "failed" },
                format!("{error:#}"),
            ),
        };
        if state == "succeeded" {
            remaining
                .retain(|old| old["id"] != item["id"] || old["reference"] != item["reference"]);
        } else if let Some(old) = remaining
            .iter_mut()
            .find(|old| old["id"] == item["id"] && old["reference"] == item["reference"])
        {
            old["retry_detail"] = json!(detail);
        }
        save_pending(ctx, &remaining)?;
        steps.push(json!({"id":item["id"],"name":item["name"],"action":action,"reference":item["reference"],"state":state,"detail":detail}));
    }
    let updated = steps
        .iter()
        .filter(|step| step["state"] == "succeeded")
        .count();
    let failed = steps.len() - updated;
    let snapshot = gather(ctx, false, scope, None)?;
    Ok(
        json!({"status":if failed == 0 {"complete"} else {"needs_attention"},"complete":failed == 0,"ready":failed == 0,
        "updated":updated,"failed":failed,"steps":steps,"snapshot":snapshot}),
    )
}

fn apply_item(
    ctx: &Context,
    item: &Value,
    offline: bool,
    job: &mut Option<&mut Job>,
) -> Result<Value> {
    let reference = text(item, "reference");
    match text(item, "action") {
        "install_docs" => {
            let (id, version) = reference
                .split_once('@')
                .context("Documentation update has no exact version")?;
            if offline || item["delivery"] == "local" {
                return crate::doc_library::install_docs_job(
                    ctx,
                    id,
                    Some(version),
                    job.as_deref_mut(),
                );
            }
            doc_updates::install_with_progress(ctx, id, Some(version), |label, done, total| {
                if let Some(job) = job.as_deref_mut() {
                    job.progress(label, done, total)?;
                }
                Ok(())
            })
        }
        "prepare_local_docs" => {
            crate::doc_library::install_docs_job(ctx, reference, None, job.as_deref_mut())
        }
        "install_template" => templates::prepare(ctx, reference, job.as_deref_mut()),
        "rebuild_template" => templates::build_latest_job(ctx, reference, job.as_deref_mut()),
        "update_image" => {
            let (id, version) = reference
                .split_once('@')
                .context("Image update has no exact version")?;
            let saved = infrastructure::execute(ctx, &json!({"command":"infra.status"}), None)?;
            // A cancelled update may already have committed its archive. Verify
            // that exact release locally instead of requiring a newer publisher
            // offer again. A missing or damaged saved archive must stay pending.
            if rows(&saved, "images").iter().any(|image| {
                image["id"] == id && image["version"] == version && image["receipt"].is_object()
            }) {
                return infrastructure::execute(
                    ctx,
                    &json!({"command":"infra.verify","id":reference,"apply_retention":true}),
                    job.as_deref_mut(),
                );
            }
            if offline {
                bail!("This image update needs a connection. Existing offline releases were kept.");
            }
            infrastructure::execute(
                ctx,
                &json!({"command":"infra.update","id":reference}),
                job.as_deref_mut(),
            )
        }
        _ => bail!("This item is not a managed content update"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Value, Value, Value, Value, Value) {
        (
            json!({"checked_at":30,"items":[
            {"id":"docs:python","kind":"docs","state":"available","action":"open_url","reference":null},
            {"id":"runtime:python","kind":"runtime","state":"available","action":"open_url","reference":null},
            {"id":"template:python/api","kind":"template","state":"available","action":"rebuild_template","reference":"python/api@1.0.0"}]}),
            json!({"checked_at":20,"packs":[
            {"id":"python","name":"Python","active_version":"1","installed_versions":["1"],"state":"available","latest":{"pack":{"version":"2"}}},
            {"id":"go","name":"Go","installed_versions":[],"state":"installable","latest":{"pack":{"version":"3"}}}]}),
            json!({"checked_at":10,"items":[
            {"id":"postgres","name":"PostgreSQL","state":"available","reference":"postgres@2"},
            {"id":"redis","name":"Redis","state":"available","reference":"redis@3"}]}),
            json!({"images":[{"id":"postgres","archived":true}]}),
            json!({"id":"swiftui","name":"SwiftUI","available":true,"installed":false,"active_version":"old-sdk","version":"new-sdk"}),
        )
    }

    #[test]
    fn one_snapshot_uses_actual_doc_channel_and_only_existing_library_families() {
        let (upstream, docs, images, vault, local) = fixture();
        let result = compose(upstream, docs, images, vault, local, &[]);
        assert_eq!(result["available_content"], 4);
        assert_eq!(result["checked_at"], 10);
        assert_eq!(result["all_checked"], true);
        let docs: Vec<_> = rows(&result, "items")
            .iter()
            .filter(|item| item["id"] == "docs:python")
            .collect();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0]["reference"], "python@2");
        assert!(!rows(&result, "items")
            .iter()
            .any(|item| item["id"] == "docs:go" || item["id"] == "image:redis"));
        assert!(!is_content_update(
            &json!({"state":"available","kind":"runtime","action":"open_url","reference":"python"})
        ));
    }

    #[test]
    fn failed_channels_never_apply_stale_offers_or_claim_a_complete_check() {
        let (upstream, mut docs, images, vault, local) = fixture();
        docs["error"] = json!("network unavailable");
        let result = compose(
            upstream,
            docs,
            images,
            vault,
            local,
            &[("images", "timeout".to_owned())],
        );
        assert_eq!(result["all_checked"], false);
        assert!(result["checked_at"].is_null());
        assert_eq!(result["available_content"], 2);
        for item in rows(&result, "items")
            .iter()
            .filter(|item| matches!(text(item, "id"), "docs:python" | "image:postgres"))
        {
            assert_eq!(item["state"], "error");
            assert!(item["action"].is_null());
        }
    }

    #[test]
    fn republished_image_is_attention_at_both_source_and_item_levels() {
        let (upstream, docs, mut images, vault, local) = fixture();
        images["items"][0]["state"] = json!("republished");
        let result = compose(upstream, docs, images, vault, local, &[]);
        assert_eq!(result["all_checked"], false);
        let source = rows(&result, "sources")
            .iter()
            .find(|item| item["id"] == "images")
            .unwrap();
        assert_eq!(source["state"], "error");
        let image = rows(&result, "items")
            .iter()
            .find(|item| item["id"] == "image:postgres")
            .unwrap();
        assert_eq!(image["state"], "error");
        assert!(image["action"].is_null());
    }

    #[test]
    fn local_reference_is_opt_in_and_current_installed_sdk_is_not_reprepared() {
        let (upstream, docs, images, vault, mut local) = fixture();
        local["installed"] = json!(true);
        let result = compose(
            upstream.clone(),
            docs.clone(),
            images.clone(),
            vault.clone(),
            local,
            &[],
        );
        assert_eq!(result["available_content"], 3);
        let result = compose(
            upstream,
            docs,
            images,
            vault,
            json!({"available":true,"installed":false}),
            &[],
        );
        assert!(!rows(&result, "items")
            .iter()
            .any(|item| item["id"] == "docs:swiftui"));
    }
}
