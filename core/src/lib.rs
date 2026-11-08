//! Shared Rust entry points for the Devline CLI and macOS bridge.

use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    path::Path,
};

mod apple_docs;
mod content_updates;
mod context;
mod devdocs;
mod devdocs_html;
mod doc_index;
mod doc_library;
mod doc_links;
mod doc_search;
mod doc_updates;
mod guides;
mod infrastructure;
mod markdown;
mod offline;
mod project;
mod project_layout;
mod project_prepare;
pub mod project_run;
mod reference;
mod service_clients;
pub mod tasks;
pub mod templates;
mod toolbox;
mod updates;

pub use context::{lock_library, safe_catalog_path, Catalog, Context, DocPack, Template};

fn text_arg<'a>(r: &'a Value, name: &str) -> Result<&'a str> {
    r.get(name)
        .and_then(Value::as_str)
        .with_context(|| format!("Missing {name}"))
}

pub fn project_context(path: &Path) -> Value {
    project::detected_context(path)
}

pub fn execute(request: &Value) -> Result<Value> {
    let ctx = Context::new(request)?;
    match text_arg(request, "command")? {
        "infra.clients" => service_clients::catalog(&ctx, request),
        "service_clients.add" => service_clients::add(&ctx, request, None),
        command if command.starts_with("infra.") => infrastructure::execute(&ctx, request, None),
        "toolbox.status" => toolbox::status(&ctx, request, false),
        "toolbox.check" => toolbox::status(&ctx, request, true),
        "toolbox.prepare" => toolbox::prepare(&ctx, request, None),
        "project.context" => project::context(&ctx, Path::new(text_arg(request, "path")?)),
        "project.readiness" => project_prepare::readiness(&ctx, request),
        "project.prepare" => project_prepare::prepare(&ctx, request, None),
        "project.runner" => project_prepare::runner(&ctx, request),
        "project.commands" => project_run::commands(&ctx, request),
        "project.start" => project_run::start(&ctx, request),
        "project.stop" => project_run::stop(&ctx, request),
        "project.status" => project_run::status(&ctx, request),
        "project.logs" => project_run::logs(&ctx, request),
        "new.ready" => project_prepare::create(&ctx, request),
        "project.pin" => project::pin(
            &ctx,
            Path::new(text_arg(request, "path")?),
            text_arg(request, "id")?,
            text_arg(request, "version")?,
        ),
        "project.unpin" => project::unpin(
            &ctx,
            Path::new(text_arg(request, "path")?),
            text_arg(request, "id")?,
        ),
        "docs.devdocs.catalog" => devdocs::catalog(request),
        "docs.devdocs.import" => devdocs::import(&ctx, request, None),
        "docs.distribution.status" => doc_updates::status(&ctx),
        "docs.local.status" => {
            if request["id"]
                .as_str()
                .is_some_and(|id| id != apple_docs::PACK_ID)
            {
                bail!("Unknown local documentation family");
            }
            apple_docs::status(&ctx)
        }
        "docs.distribution.check" => doc_updates::check(&ctx),
        "docs.distribution.install" => doc_updates::install(&ctx, request),
        "docs.distribution.rollback" => doc_updates::rollback(&ctx, request),
        "docs.distribution.configure" => doc_updates::configure(&ctx, text_arg(request, "url")?),
        "jobs.start" => tasks::start(&ctx, request),
        "jobs.status" => tasks::status(&ctx, request),
        "jobs.cancel" => tasks::cancel(&ctx, request),
        "jobs.list" => tasks::list(&ctx),
        "offline.status" => offline::status(&ctx, request),
        "offline.check" => templates::verify_offline(&ctx, text_arg(request, "id")?, None),
        "bundle.export" => offline::export(&ctx, request),
        "bundle.inspect" => offline::inspect(&ctx, request),
        "bundle.import" => offline::import(&ctx, request),
        "templates.inspect" => templates::inspect(&ctx, text_arg(request, "id")?),
        "templates.remove" => templates::remove(&ctx, text_arg(request, "id")?),
        "catalog" => {
            let db = ctx.database()?;
            let docs = doc_library::catalog(&ctx, &db)?;
            Ok(json!({"docs":docs,"templates":templates::list(&ctx)?,"home":ctx.home}))
        }
        "docs.install" => {
            doc_library::install_docs(&ctx, text_arg(request, "id")?, request["version"].as_str())
        }
        "docs.import.local" => doc_library::import_local(
            &ctx,
            text_arg(request, "id")?,
            text_arg(request, "version")?,
            request["keep_latest"] == true,
        ),
        "docs.search" => doc_search::search(&ctx, request),
        "docs.navigation" => doc_search::navigation(&ctx, request),
        "docs.show" => doc_search::show(&ctx, request),
        "docs.resolve" => doc_links::resolve(&ctx, request),
        "docs.link-status" => doc_links::status(&ctx, request),
        "guide.list" => guides::list(&ctx),
        "guide.show" => guides::show(&ctx, text_arg(request, "id")?),
        "reading.list" => reference::list(&ctx),
        "reading.visit" => reference::visit(
            &ctx,
            text_arg(request, "id")?,
            request["anchor"].as_str(),
            request["focus_text"].as_str(),
        ),
        "reading.save" => reference::save(
            &ctx,
            text_arg(request, "id")?,
            request["anchor"].as_str(),
            request["focus_text"].as_str(),
        ),
        "reading.remove" => reference::remove(&ctx, text_arg(request, "id")?),
        "library.retention" => updates::retention(&ctx, request),
        "library.purge" => {
            let _lock = lock_library(&ctx)?;
            let docs = doc_updates::prune(&ctx, None)?;
            let starters = templates::prune(&ctx, None)?;
            drop(_lock);
            let images = infrastructure::prune(&ctx)?;
            Ok(json!({"docs":docs,"starters":starters,"images":images}))
        }
        "docs.remove" => doc_library::remove(&ctx, request),
        "templates.list" => templates::list(&ctx),
        "runtime.list" => templates::runtimes(&ctx),
        "updates.app.status" => updates::app_status(&ctx),
        "updates.app.check" => updates::app_check(&ctx, request),
        "updates.status" => content_updates::status(&ctx),
        "updates.runtime.status" => updates::runtime_status(&ctx),
        "updates.check" => content_updates::check(&ctx, request, None),
        "updates.apply" => content_updates::apply(&ctx, request, None),
        "updates.configure" => updates::configure(&ctx, text_arg(request, "url")?),
        "templates.install" => templates::install(&ctx, text_arg(request, "id")?),
        "doctor" => templates::doctor(&ctx, text_arg(request, "id")?),
        "new" => templates::create(
            &ctx,
            text_arg(request, "id")?,
            Path::new(text_arg(request, "destination")?),
            request["dry_run"].as_bool().unwrap_or(false),
        ),
        "templates.build" => templates::build_latest(&ctx, text_arg(request, "id")?),
        "prepare" => templates::prepare(&ctx, text_arg(request, "id")?, None),
        other => bail!("Unknown command {other}"),
    }
}
/// Execute a JSON request and return an owned, NUL-terminated JSON response.
///
/// # Safety
/// A non-null `input` must point to a readable NUL-terminated string for the
/// duration of this call. Release the returned pointer exactly once with
/// `devline_free`; do not deallocate it with a foreign allocator.
#[no_mangle]
pub unsafe extern "C" fn devline_request(input: *const c_char) -> *mut c_char {
    let response = std::panic::catch_unwind(|| -> Result<Value> {
        if input.is_null() {
            bail!("Empty request");
        }
        let request: Value = serde_json::from_str(CStr::from_ptr(input).to_str()?)?;
        execute(&request)
    });
    let value = match response {
        Ok(Ok(data)) => json!({"ok":true,"data":data}),
        Ok(Err(e)) => json!({"ok":false,"error":format!("{e:#}")}),
        Err(_) => json!({"ok":false,"error":"Internal error. The operation could not finish."}),
    };
    CString::new(value.to_string()).unwrap().into_raw()
}
/// Release a response allocated by `devline_request`.
///
/// # Safety
/// `pointer` must be null or an unmodified pointer returned by `devline_request`
/// that has not already been freed. The response must no longer be in use.
#[no_mangle]
pub unsafe extern "C" fn devline_free(pointer: *mut c_char) {
    if !pointer.is_null() {
        drop(CString::from_raw(pointer));
    }
}
