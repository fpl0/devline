use crate::{infrastructure, offline, Context};
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::collections::BTreeSet;

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
    Ok(
        json!({"path":path,"ready_offline":ready,"status":status,"library":library,
        "infrastructure":stack,"docker":docker,"images":images,"project_dependencies":project_dependencies}),
    )
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
