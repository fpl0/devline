use crate::{markdown, safe_catalog_path, valid_doc_id, Context};
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::{fs, io::Read};

const DIRECTORY: &str = "help/infrastructure";
const SOURCE: &str = "https://github.com/fpl0/devline/blob/main/data/help/infrastructure";
const MAX_BYTES: u64 = 256 * 1024;

fn page(ctx: &Context, id: &str) -> Result<(String, String)> {
    if !valid_doc_id(id) {
        bail!("Choose a Devline guide ID from devline guide");
    }
    let path = safe_catalog_path(ctx, &format!("{DIRECTORY}/{id}.md")).with_context(|| {
        format!("Devline guide {id:?} is unavailable. Run devline guide to list available guides.")
    })?;
    let file = fs::File::open(&path)?;
    if !file.metadata()?.is_file() {
        bail!("Invalid Devline guide {id:?}");
    }
    let mut text = String::new();
    file.take(MAX_BYTES + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_BYTES {
        bail!("Devline guide {id:?} exceeds the supported size");
    }
    let (heading, body) = text.split_once('\n').context("Missing guide title")?;
    let title = heading
        .strip_prefix("# ")
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .context("Missing guide title")?;
    Ok((title.to_owned(), body.trim().to_owned()))
}

pub(super) fn list(ctx: &Context) -> Result<Value> {
    if !ctx.catalog_dir.join(DIRECTORY).try_exists()? {
        return Ok(json!([]));
    }
    let directory = safe_catalog_path(ctx, DIRECTORY)?;
    let mut guides = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let Some(id) = path
            .file_stem()
            .and_then(|id| id.to_str())
            .filter(|id| valid_doc_id(id))
        else {
            continue;
        };
        let (title, _) = page(ctx, id)?;
        guides.push((id.to_owned(), title));
    }
    guides.sort_unstable();
    Ok(json!(guides
        .into_iter()
        .map(|(id, title)| json!({"id":id,"title":title}))
        .collect::<Vec<_>>()))
}

pub(super) fn show(ctx: &Context, id: &str) -> Result<Value> {
    let (title, body) = page(ctx, id)?;
    let source_url = format!("{SOURCE}/{id}.md");
    let (html, _) = markdown::render(&body, &source_url, None);
    Ok(json!({"id":id,"title":title,"body":body,"html":html,"source_url":source_url}))
}
