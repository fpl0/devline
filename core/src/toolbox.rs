use crate::{infrastructure, offline, project_prepare, tasks::Job, templates, Context};
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};

pub(super) fn status(ctx: &Context, request: &Value, verify_images: bool) -> Result<Value> {
    let mut releases = image_selection(request)?;
    let library = offline::status(ctx, request)?;
    let project = &library["project"];
    let path = project["path"].as_str();
    let stack = match path {
        Some(path) => infrastructure::execute(
            ctx,
            &json!({"command":"infra.stack.config","path":path}),
            None,
        )?,
        None => Value::Null,
    };
    for service in rows(&stack, "services") {
        let id = service["id"]
            .as_str()
            .context("Project service is missing its identity")?;
        let version = service["version"]
            .as_str()
            .context("Project service is missing its version")?;
        releases.insert(format!("{id}@{version}"));
    }
    let (docker, images) = archive_status(ctx, releases, verify_images)?;
    let proof = path
        .map(|_| crate::project_prepare::readiness_for_profile(ctx, project))
        .transpose()?;
    let project_dependencies = project_dependencies(proof.as_ref());
    let library_selected = library["status"] != "no_selection";
    let selected = library_selected || !images.is_empty() || path.is_some();
    let ready = selected
        && (!library_selected || library["ready_offline"] == true)
        && images.iter().all(|image| image["ready_offline"] == true)
        && project_dependencies["state"] != "unverified";
    let status = if ready {
        "ready_offline"
    } else if selected {
        "needs_preparation"
    } else {
        "no_selection"
    };
    let mut result = json!({"path":path,"ready_offline":ready,"status":status,"library":library,
        "infrastructure":stack,"docker":docker,"images":images,"project_dependencies":project_dependencies});
    result["blockers"] = json!(blockers(&result));
    Ok(result)
}

fn blockers(value: &Value) -> Vec<Value> {
    let mut result = Vec::new();
    for (key, kind, action) in [
        ("docs", "documentation", "Prepare matching documentation"),
        (
            "runtimes",
            "runtime",
            "Install or select the required toolchain using its own tools",
        ),
        ("starters", "starter", "Prepare the selected starter"),
    ] {
        for row in rows(&value["library"], key).filter(|row| row["ready_offline"] != true) {
            result.push(json!({"kind":kind,"id":row["id"],"detail":row["detail"].as_str().or(row["error"].as_str()).unwrap_or("This resource needs preparation."),"action":action}));
        }
    }
    if value["project_dependencies"]["state"] == "unverified" {
        result.push(json!({"kind":"project","id":value["path"],"detail":value["project_dependencies"]["detail"],"action":"Prepare and verify this project's dependencies"}));
    }
    for image in rows(value, "images").filter(|row| row["ready_offline"] != true) {
        let action = if image["status"] == "needs_docker" {
            "Start a compatible local Docker engine"
        } else {
            "Prepare the pinned image archive"
        };
        result.push(json!({"kind":"image","id":format!("{}@{}",image["id"].as_str().unwrap_or_default(),image["version"].as_str().unwrap_or_default()),"detail":image["detail"].as_str().unwrap_or(action),"action":action}));
    }
    result
}

fn stage(job: &mut Option<&mut Job>, label: &str) -> Result<()> {
    if let Some(job) = job.as_deref_mut() {
        job.stage(label)?;
    }
    Ok(())
}

fn outcome(
    steps: &mut Vec<Value>,
    job: &Option<&mut Job>,
    kind: &str,
    id: &str,
    result: Result<Value>,
) -> Result<()> {
    // An individual resource failure is useful evidence; cancellation and timeouts
    // stop the entire job before another resource is changed.
    if let Some(job) = job.as_deref() {
        job.checkpoint()?;
    }
    steps.push(match result {
        Ok(result) => json!({"kind":kind,"id":id,"status":"prepared","result":result}),
        Err(error) => json!({"kind":kind,"id":id,"status":"blocked","detail":format!("{error:#}")}),
    });
    Ok(())
}

pub(super) fn prepare(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let path = Path::new(crate::text_arg(request, "path")?);
    let root = project_prepare::mutation_root(ctx, path)?;
    let mut request = request.clone();
    request["path"] = json!(root);
    let offline = request["offline"] == true;
    stage(&mut job, "Waiting for this project's preparation lock")?;
    let _lock = project_prepare::lock_project(ctx, &root, &job)?;
    project_prepare::recover_locked(ctx, &root)?;
    let mut initial = status(ctx, &request, false)?;
    let mut steps = Vec::new();

    // Missing references can be discovered from the official distribution channel
    // while connected. Existing compatible selections and pins never move.
    let local_docs: BTreeSet<_> = ctx
        .catalog
        .docs
        .iter()
        .map(|pack| (pack.id.clone(), pack.version.clone()))
        .chain(
            crate::doc_updates::cached_packs(ctx)?
                .into_iter()
                .map(|(pack, _)| (pack.id, pack.version)),
        )
        .collect();
    let unresolved = rows(&initial["library"], "docs")
        .filter(|doc| doc["ready_offline"] != true && doc["id"] != "swiftui")
        .any(|doc| {
            let id = doc["id"].as_str().unwrap_or_default();
            let selected = rows(&initial["library"]["project"], "docsets")
                .find(|selected| selected["id"] == id);
            let version = doc["version"]
                .as_str()
                .or_else(|| selected.and_then(|selected| selected["prepare_version"].as_str()));
            version.is_none_or(|version| !local_docs.contains(&(id.to_owned(), version.to_owned())))
        });
    if unresolved && !offline {
        stage(
            &mut job,
            "Checking the official documentation channel for missing references",
        )?;
        let result = crate::doc_updates::check(ctx);
        outcome(&mut steps, &job, "channel", "documentation", result)?;
        initial = status(ctx, &request, false)?;
    }

    // Explicit library selections remain required. Project provenance does not.
    let starters = request["starters"].as_array().cloned().unwrap_or_default();
    for reference in starters {
        let reference = reference
            .as_str()
            .context("A starter reference must be a string")?;
        stage(&mut job, &format!("Preparing starter {reference}"))?;
        let result = templates::prepare(ctx, reference, job.as_deref_mut());
        outcome(&mut steps, &job, "starter", reference, result)?;
    }

    let profile = &initial["library"]["project"];
    let mut docs = BTreeSet::new();
    for row in rows(&initial["library"], "docs").filter(|row| row["ready_offline"] != true) {
        let id = crate::text_arg(row, "id")?;
        let selected = rows(profile, "docsets").find(|doc| doc["id"] == id);
        let version = row["version"]
            .as_str()
            .or_else(|| selected.and_then(|doc| doc["prepare_version"].as_str()));
        if let Some(version) = version {
            docs.insert((id.to_owned(), Some(version.to_owned())));
        } else if id == "swiftui" || selected.is_none() {
            docs.insert((id.to_owned(), None));
        } else {
            steps.push(
                json!({"kind":"documentation","id":id,"status":"blocked","detail":row["detail"]}),
            );
        }
    }
    for (id, version) in docs {
        let reference = version
            .as_ref()
            .map_or_else(|| id.clone(), |version| format!("{id}@{version}"));
        stage(&mut job, &format!("Preparing documentation {reference}"))?;
        let mut result =
            crate::doc_library::install_docs_job(ctx, &id, version.as_deref(), job.as_deref_mut());
        if result.is_err() && !offline && id != "swiftui" {
            if let Some(job) = job.as_deref() {
                job.checkpoint()?;
            }
            result = crate::doc_updates::install_with_progress(
                ctx,
                &id,
                version.as_deref(),
                |label, done, total| {
                    if let Some(job) = job.as_deref_mut() {
                        job.progress(label, done, total)?;
                    }
                    Ok(())
                },
            );
        }
        outcome(&mut steps, &job, "documentation", &reference, result)?;
    }

    for image in rows(&initial, "images") {
        let reference = format!(
            "{}@{}",
            crate::text_arg(image, "id")?,
            crate::text_arg(image, "version")?
        );
        stage(&mut job, &format!("Verifying image archive {reference}"))?;
        let mut result = infrastructure::execute(
            ctx,
            &json!({"command":"infra.verify","id":reference}),
            job.as_deref_mut(),
        );
        if result.is_err() && !offline {
            if let Some(job) = job.as_deref() {
                job.checkpoint()?;
            }
            stage(&mut job, &format!("Preparing pinned image {reference}"))?;
            result = infrastructure::execute(
                ctx,
                &json!({"command":"infra.prepare","id":reference}),
                job.as_deref_mut(),
            );
        }
        outcome(&mut steps, &job, "image", &reference, result)?;
    }

    if offline || initial["project_dependencies"]["state"] != "verified_project" {
        // Source edits invalidate the receipt, not the dependency closure. Always
        // try the existing project with networking denied before fetching again.
        stage(
            &mut job,
            "Testing existing project dependencies with networking denied",
        )?;
        let mut local = request.clone();
        local["offline"] = json!(true);
        let mut result = project_prepare::prepare_locked(ctx, &root, &local, job.as_deref_mut());
        if result.is_err() && !offline {
            if let Some(job) = job.as_deref() {
                job.checkpoint()?;
            }
            stage(
                &mut job,
                "Preparing locked dependencies after local verification failed",
            )?;
            result = project_prepare::prepare_locked(ctx, &root, &request, job.as_deref_mut());
        }
        outcome(
            &mut steps,
            &job,
            "project",
            crate::text_arg(&request, "path")?,
            result,
        )?;
    } else {
        steps.push(json!({"kind":"project","id":request["path"],"status":"current","detail":"Current files and toolchain already have a valid offline verification receipt."}));
    }
    stage(&mut job, "Checking the complete project toolbox")?;
    let mut result = status(ctx, &request, false)?;
    // Failed attempts remain visible even when another component prepared successfully.
    let mut blockers = result["blockers"].as_array().cloned().unwrap_or_default();
    for step in steps
        .iter()
        .filter(|step| step["status"] == "blocked" && step["kind"] != "channel")
    {
        if !blockers
            .iter()
            .any(|blocker| blocker["kind"] == step["kind"] && blocker["id"] == step["id"])
        {
            blockers.push(step.clone());
        }
    }
    if !blockers.is_empty() {
        result["ready_offline"] = json!(false);
        result["status"] = json!("needs_preparation");
    }
    result["blockers"] = json!(blockers);
    result["steps"] = json!(steps);
    result["prepared"] = result["ready_offline"].clone();
    Ok(result)
}

fn rows<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    value[key].as_array().into_iter().flatten()
}

fn image_selection(request: &Value) -> Result<BTreeSet<String>> {
    let Some(images) = request.get("images") else {
        return Ok(BTreeSet::new());
    };
    let images = images
        .as_array()
        .context("Select images as a list of exact SERVICE@VERSION releases")?;
    if images.len() > 100 {
        bail!("Select at most 100 image releases");
    }
    let mut releases = BTreeSet::new();
    for image in images {
        let id = image
            .as_str()
            .context("An image release must be a string")?;
        let (service, version) = id
            .split_once('@')
            .context("Use an exact image release (SERVICE@VERSION)")?;
        if service.is_empty()
            || service.len() > 128
            || !service
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || version.is_empty()
            || version.len() > 128
            || !version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))
        {
            bail!("Use an exact image release (SERVICE@VERSION)");
        }
        releases.insert(id.to_owned());
    }
    Ok(releases)
}

fn archive_status(
    ctx: &Context,
    releases: BTreeSet<String>,
    verify: bool,
) -> Result<(Value, Vec<Value>)> {
    if releases.is_empty() {
        return Ok((Value::Null, Vec::new()));
    }
    let available = infrastructure::execute(ctx, &json!({"command":"infra.status"}), None)?;
    let docker = &available["docker"];
    let mut images = Vec::with_capacity(releases.len());
    for reference in releases {
        let (id, version) = reference
            .split_once('@')
            .expect("validated image reference");
        let local = rows(&available, "images").find(|row| {
            row["id"] == id
                && row["version"] == version
                && (available["platform"].is_null() || row["platform"] == available["platform"])
        });
        let mut row = local.cloned().unwrap_or_else(|| json!({"id":id,"version":version,"platform":available["platform"].as_str().unwrap_or("unknown"),"archived":false,
            "ready_offline":false,"status":"needs_archive","detail":"No local archive for this release on this platform."}));
        if verify && row["archived"] == true {
            match infrastructure::execute(
                ctx,
                &json!({"command":"infra.verify","id":reference}),
                None,
            ) {
                Ok(result) => {
                    row["receipt"] = result["receipt"].clone();
                    row["integrity"] = result["integrity"].clone();
                    row["archived"] = result["archived"].clone();
                    row["ready_offline"] = json!(
                        result["integrity"] == "verified"
                            && docker["available"] == true
                            && row["platform"] == docker["platform"]
                    );
                    row["status"] = json!(if row["ready_offline"] == true {
                        "ready_offline"
                    } else {
                        "needs_docker"
                    });
                }
                Err(error) => {
                    row["integrity"] = json!("failed");
                    row["ready_offline"] = json!(false);
                    row["status"] = json!("needs_verification");
                    row["detail"] = json!(format!("{error:#}"));
                }
            }
        }
        images.push(row);
    }
    Ok((docker.clone(), images))
}

fn project_dependencies(proof: Option<&Value>) -> Value {
    let Some(proof) = proof else {
        return json!({"state":"not_selected"});
    };
    let state = if proof["ready_offline"] == true {
        "verified_project"
    } else {
        "unverified"
    };
    json!({"state":state,"detail":proof["detail"],"verified_at":proof["verified_at"]})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_readiness_requires_matching_test_evidence() {
        assert_eq!(project_dependencies(None)["state"], "not_selected");
        assert_eq!(
            project_dependencies(Some(&json!({"ready_offline":false})))["state"],
            "unverified"
        );
        assert_eq!(
            project_dependencies(Some(&json!({"ready_offline":true})))["state"],
            "verified_project"
        );
    }

    #[test]
    fn image_selection_rejects_unpinned_and_deduplicates_pinned_releases() {
        assert!(image_selection(&json!({"images":["postgres"]})).is_err());
        let selected =
            image_selection(&json!({"images":["postgres@18.1","postgres@18.1"]})).unwrap();
        assert_eq!(selected.len(), 1);
    }
}
