use crate::{progress, render_offline, request, run_operation, s, shell_quote, terminal, Cli};
use anyhow::{bail, Result};
use clap::{Args, Subcommand, ValueHint};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

type Output = (&'static str, Value);

#[derive(Subcommand)]
pub(super) enum Infra {
    #[command(about = "Browse the supported container images and exact releases")]
    Catalog,
    #[command(about = "Show recommended optional client libraries for a project or runtime")]
    Clients {
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        #[arg(long)]
        stack: Option<String>,
    },
    #[command(about = "Read cached image updates or explicitly check publishers online")]
    Updates {
        #[command(subcommand)]
        command: Option<UpdateStatus>,
    },
    #[command(about = "Archive a checked image update alongside the current release")]
    Update {
        #[arg(value_name = "SERVICE[@VERSION]", required_unless_present = "all", conflicts_with = "all", value_parser = update_reference)]
        id: Option<String>,
        /// Archive all available updates from the last publisher check.
        #[arg(long)]
        all: bool,
        #[arg(long)]
        background: bool,
    },
    #[command(about = "Show the image vault or a project's local containers")]
    Status {
        /// Image release to inspect, such as postgres@18.1.
        #[arg(value_name = "SERVICE@VERSION", value_parser = release, conflicts_with = "project")]
        id: Option<String>,
        /// Show the stack in this project directory.
        #[arg(long, value_hint = ValueHint::DirPath)]
        project: Option<PathBuf>,
    },
    #[command(about = "Download and verify an exact image while connected")]
    Prepare(ImageJob),
    #[command(about = "Load an archived image into local Docker without pulling")]
    Restore(ImageJob),
    #[command(about = "Verify a stored image archive without Docker or a connection")]
    Verify(ImageJob),
    #[command(about = "Remove an exact release from the local image vault")]
    Remove {
        #[arg(value_name = "SERVICE@VERSION", value_parser = release)]
        id: String,
    },
    #[command(about = "Export verified image archives to a portable vault bundle")]
    Export {
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,
        /// Exact image release to include. Repeat to include more.
        #[arg(long = "image", value_name = "SERVICE@VERSION", value_parser = release, required = true)]
        images: Vec<String>,
        #[arg(long)]
        background: bool,
    },
    #[command(about = "Validate and import images from a local vault bundle")]
    Import {
        #[arg(value_hint = ValueHint::FilePath)]
        path: PathBuf,
        /// Preview the archive and any publisher trust requirement.
        #[arg(long, conflicts_with_all = ["trust_images", "manifest_sha256", "background"])]
        inspect: bool,
        /// Import compatible image pins after reviewing the bundle source.
        #[arg(long)]
        trust_images: bool,
        /// Require the manifest SHA-256 from the inspected archive.
        #[arg(long, value_name = "HEX", value_parser = manifest_digest)]
        manifest_sha256: Option<String>,
        #[arg(long)]
        background: bool,
    },
    #[command(about = "Save exact services and optional local ports for a project")]
    Configure {
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
        /// Exact service release. Repeat to include more.
        #[arg(long = "service", value_name = "SERVICE@VERSION", value_parser = release, required = true)]
        services: Vec<String>,
        /// Override a published localhost port. Repeat for multiple ports.
        #[arg(long = "port", value_name = "SERVICE:PORT=HOST_PORT", value_parser = port)]
        ports: Vec<Port>,
        /// Explicitly add a recommended runtime client and prepare its dependencies.
        #[arg(long = "client", value_name = "STACK:SERVICE")]
        clients: Vec<String>,
    },
    #[command(about = "Add services without changing existing pins, ports, credentials, or data")]
    Add {
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
        /// Exact service release. Repeating an existing selection is safe.
        #[arg(long = "service", value_name = "SERVICE@VERSION", value_parser = release, required_unless_present = "clients")]
        services: Vec<String>,
        /// Choose a localhost port for a newly added service.
        #[arg(long = "port", value_name = "SERVICE:PORT=HOST_PORT", value_parser = port)]
        ports: Vec<Port>,
        /// Add only these explicitly selected client dependencies; repeat for more.
        #[arg(long = "client", value_name = "STACK:SERVICE")]
        clients: Vec<String>,
        #[arg(long)]
        background: bool,
    },
    #[command(
        visible_alias = "up",
        about = "Start a project's services using archived images, without pulling"
    )]
    Run(StackJob),
    #[command(
        visible_alias = "down",
        about = "Stop a project's services and keep their data"
    )]
    Stop {
        #[command(flatten)]
        job: StackJob,
        /// Also delete this stack's data volumes.
        #[arg(long)]
        delete_volumes: bool,
    },
    #[command(about = "Read recent output from a project's services")]
    Logs {
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..=500))]
        tail: u32,
    },
    #[command(about = "Show this project's generated local credentials explicitly")]
    Credentials {
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
    },
    #[command(
        about = "Print shell exports for local client connections (includes credentials)",
        after_long_help = "Use eval \"$(devline infra env .)\" before running your app. Exports include DEVLINE_<SERVICE>_URL and supported .NET ConnectionStrings__ keys. Nothing is written to your project or shell automatically."
    )]
    Env {
        #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
        path: PathBuf,
    },
}

#[derive(Args)]
pub(super) struct ImageJob {
    #[arg(value_name = "SERVICE@VERSION", value_parser = release)]
    id: String,
    /// Start a cancellable job and print its ID.
    #[arg(long)]
    background: bool,
}

#[derive(Subcommand)]
pub(super) enum UpdateStatus {
    #[command(about = "Check publisher registries for compatible, stable releases")]
    Check,
}

#[derive(Args)]
pub(super) struct StackJob {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
    /// Start a cancellable job and print its ID.
    #[arg(long)]
    background: bool,
}

#[derive(Clone)]
pub(super) struct Port {
    key: String,
    host: u16,
}

fn service(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub(super) fn release(value: &str) -> Result<String, String> {
    if let Some((id, version)) = value.split_once('@') {
        if service(id)
            && !version.is_empty()
            && version.len() <= 128
            && version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))
        {
            return Ok(value.to_owned());
        }
    }
    Err("Use an exact image release from `devline infra catalog` (SERVICE@VERSION).".into())
}

fn update_reference(value: &str) -> Result<String, String> {
    if service(value) {
        return Ok(value.into());
    }
    release(value)
}

fn manifest_digest(value: &str) -> Result<String, String> {
    if value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(value.to_ascii_lowercase());
    }
    Err("Use the 64-digit manifest SHA-256 from `infra import --inspect`.".into())
}

pub(super) fn port(value: &str) -> Result<Port, String> {
    if let Some((key, host)) = value.split_once('=') {
        if key
            .split_once(':')
            .is_some_and(|(id, port)| service(id) && service(port))
        {
            if let Ok(host) = host.parse::<u16>() {
                if host != 0 {
                    return Ok(Port {
                        key: key.into(),
                        host,
                    });
                }
            }
        }
    }
    Err("Use SERVICE:PORT=HOST_PORT with a host port between 1 and 65535.".into())
}

pub(super) fn port_overrides(ports: &[Port]) -> Result<BTreeMap<&str, u16>> {
    let mut overrides = BTreeMap::new();
    for port in ports {
        if overrides.insert(port.key.as_str(), port.host).is_some() {
            bail!("Port {} was specified more than once.", port.key);
        }
    }
    Ok(overrides)
}

#[derive(Args)]
pub(super) struct Toolbox {
    #[command(subcommand)]
    command: Option<ToolboxAction>,
    /// Project directory. Defaults to the current directory.
    #[arg(long, global = true, conflicts_with = "library_only", value_hint = ValueHint::DirPath)]
    path: Option<PathBuf>,
    /// Check only the selected library releases, without assuming a project.
    #[arg(long, global = true)]
    library_only: bool,
    /// Include an exact documentation release. Repeat to include more.
    #[arg(long = "doc", global = true, value_name = "PACK@VERSION")]
    docs: Vec<String>,
    /// Include an exact starter release. Repeat to include more.
    #[arg(long = "starter", global = true, value_name = "STARTER@VERSION")]
    starters: Vec<String>,
    /// Include an exact image release. Repeat to include more.
    #[arg(long = "image", global = true, value_name = "SERVICE@VERSION", value_parser = release)]
    images: Vec<String>,
}

#[derive(Subcommand)]
enum ToolboxAction {
    #[command(about = "Report this project's offline readiness and archived images")]
    Status,
    #[command(about = "Also verify selected image files from disk")]
    Check,
    #[command(
        about = "Prepare this project's references, dependencies and pinned images together"
    )]
    Prepare {
        #[arg(long)]
        background: bool,
    },
}

pub(super) fn run(cli: &Cli, command: Option<&Infra>) -> Result<Output> {
    match command {
        None | Some(Infra::Catalog) => Ok((
            "infra-catalog",
            request(cli, json!({"command":"infra.catalog"}))?,
        )),
        Some(Infra::Clients { path, stack }) => {
            let path = if path.is_none() && stack.is_none() {
                Some(std::env::current_dir()?)
            } else {
                path.clone()
            };
            Ok((
                "infra-clients",
                request(
                    cli,
                    json!({"command":"infra.clients","path":path,"stack":stack}),
                )?,
            ))
        }
        Some(Infra::Updates { command }) => {
            let checking = matches!(command, Some(UpdateStatus::Check));
            if checking && cli.offline {
                bail!("Checking image publishers requires a connection. Use `devline infra updates` for saved results.");
            }
            let operation = if checking {
                "infra.updates.check"
            } else {
                "infra.updates"
            };
            if checking {
                progress(
                    cli,
                    format_args!("Checking compatible releases from image publishers…"),
                );
            }
            Ok(("infra-updates", request(cli, json!({"command":operation}))?))
        }
        Some(Infra::Update {
            id,
            all,
            background,
        }) => {
            if cli.offline {
                bail!("Archiving an image update requires a connection. Use `devline infra restore SERVICE@VERSION` for an existing archive.");
            }
            if !all {
                return run_operation(
                    cli,
                    "infra-image",
                    "infra_update",
                    *background,
                    json!({"command":"infra.update","id":id}),
                );
            }
            let updates = request(cli, json!({"command":"infra.updates"}))?;
            if updates["checked_at"].is_null() {
                bail!("Run `devline infra updates check` first.");
            }
            let mut results = Vec::new();
            let mut failed = 0;
            for reference in items(&updates, "items")
                .filter(|item| item["state"] == "available")
                .filter_map(|item| item["reference"].as_str())
            {
                progress(cli, format_args!("Archiving {reference}…"));
                let result = run_operation(
                    cli,
                    "infra-image",
                    "infra_update",
                    *background,
                    json!({"command":"infra.update","id":reference}),
                );
                match result {
                    Ok((_, result)) => results.push(json!({"reference":reference,"result":result})),
                    Err(error) => {
                        failed += 1;
                        results.push(json!({"reference":reference,"error":format!("{error:#}")}));
                    }
                }
            }
            Ok((
                "infra-update-batch",
                json!({"items":results,"failed":failed,"background":background}),
            ))
        }
        Some(Infra::Status { id, project }) => {
            if let Some(path) = project {
                return stack(cli, "infra-stack", "infra.stack.status", path);
            }
            Ok((
                "infra-status",
                request(cli, json!({"command":"infra.status","id":id}))?,
            ))
        }
        Some(Infra::Prepare(job)) => {
            if cli.offline {
                bail!("Preparing an image can download it. Use `devline infra restore {}` for a local archive, or `devline infra import <bundle>`.", job.id);
            }
            progress(
                cli,
                format_args!("Preparing {} from its pinned upstream image…", job.id),
            );
            image_job(cli, "infra_prepare", "infra.prepare", job)
        }
        Some(Infra::Restore(job)) => image_job(cli, "infra_restore", "infra.restore", job),
        Some(Infra::Verify(job)) => image_job(cli, "infra_verify", "infra.verify", job),
        Some(Infra::Remove { id }) => Ok((
            "infra-removed",
            request(cli, json!({"command":"infra.remove","id":id}))?,
        )),
        Some(Infra::Export {
            path,
            images,
            background,
        }) => run_operation(
            cli,
            "infra-exported",
            "infra_export",
            *background,
            json!({"command":"infra.export","path":path,"images":images}),
        ),
        Some(Infra::Import {
            path,
            inspect: true,
            ..
        }) => Ok((
            "infra-import-inspection",
            request(cli, json!({"command":"infra.import.inspect","path":path}))?,
        )),
        Some(Infra::Import {
            path,
            trust_images,
            manifest_sha256,
            background,
            ..
        }) => run_operation(
            cli,
            "infra-imported",
            "infra_import",
            *background,
            json!({"command":"infra.import","path":path,"trust_images":trust_images,"manifest_sha256":manifest_sha256}),
        ),
        Some(Infra::Configure {
            path,
            services,
            ports,
            clients,
        }) => {
            let overrides = port_overrides(ports)?;
            Ok((
                "infra-configured",
                request(
                    cli,
                    json!({"command":"infra.stack.create","path":path,"services":services,"ports":overrides,"clients":clients,"offline":cli.offline}),
                )?,
            ))
        }
        Some(Infra::Add {
            path,
            services,
            ports,
            clients,
            background,
        }) => {
            let overrides = port_overrides(ports)?;
            if services.is_empty() {
                if !overrides.is_empty() {
                    bail!("Port overrides require a selected service.");
                }
                run_operation(
                    cli,
                    "infra-clients-added",
                    "service_clients_add",
                    *background,
                    json!({"command":"service_clients.add","path":path,"clients":clients,"offline":cli.offline}),
                )
            } else {
                run_operation(
                    cli,
                    "infra-configured",
                    "infra_stack_add",
                    *background,
                    json!({"command":"infra.stack.add","path":path,"services":services,"ports":overrides,"clients":clients,"offline":cli.offline}),
                )
            }
        }
        Some(Infra::Run(job)) => run_operation(
            cli,
            "infra-running",
            "infra_stack_up",
            job.background,
            json!({"command":"infra.stack.up","path":job.path}),
        ),
        Some(Infra::Stop {
            job,
            delete_volumes,
        }) => run_operation(
            cli,
            "infra-stopped",
            "infra_stack_down",
            job.background,
            json!({"command":"infra.stack.down","path":job.path,"delete_volumes":delete_volumes}),
        ),
        Some(Infra::Logs { path, tail }) => Ok((
            "infra-logs",
            request(
                cli,
                json!({"command":"infra.stack.logs","path":path,"tail":tail}),
            )?,
        )),
        Some(Infra::Credentials { path }) => {
            stack(cli, "infra-credentials", "infra.stack.credentials", path)
        }
        Some(Infra::Env { path }) => stack(cli, "infra-env", "infra.stack.credentials", path),
    }
}

fn image_job(cli: &Cli, kind: &str, command: &str, job: &ImageJob) -> Result<Output> {
    run_operation(
        cli,
        "infra-image",
        kind,
        job.background,
        json!({"command":command,"id":job.id}),
    )
}

fn stack(cli: &Cli, action: &'static str, command: &str, path: &Path) -> Result<Output> {
    Ok((
        action,
        request(cli, json!({"command":command,"path":path}))?,
    ))
}

pub(super) fn project_config(cli: &Cli, path: &Path) -> Result<Value> {
    request(cli, json!({"command":"infra.stack.config","path":path}))
}

pub(super) fn toolbox(cli: &Cli, options: &Toolbox) -> Result<Output> {
    let path = if options.library_only {
        None
    } else {
        Some(
            options
                .path
                .clone()
                .map_or_else(std::env::current_dir, Ok)?,
        )
    };
    if let Some(ToolboxAction::Prepare { background }) = options.command {
        if options.library_only {
            bail!("Toolbox preparation needs a project. Use --path <folder> or run it in the project directory.");
        }
        return run_operation(
            cli,
            "toolbox",
            "toolbox_prepare",
            background,
            json!({"command":"toolbox.prepare","path":path,"offline":cli.offline,"docs":options.docs,"starters":options.starters,"images":options.images}),
        );
    }
    let command = if matches!(options.command, Some(ToolboxAction::Check)) {
        progress(
            cli,
            format_args!("Checking local project resources and image archives…"),
        );
        "toolbox.check"
    } else {
        "toolbox.status"
    };
    Ok((
        "toolbox",
        request(
            cli,
            json!({"command":command,"path":path,"docs":options.docs,
        "starters":options.starters,"images":options.images}),
        )?,
    ))
}

fn items<'a>(value: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    value[key].as_array().into_iter().flatten()
}

fn reference(value: &Value) -> String {
    value["reference"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}@{}", s(value, "id"), s(value, "version")))
}

pub(super) fn complete_images(cli: &Cli) -> Result<()> {
    let catalog = request(cli, json!({"command":"infra.catalog"}))?;
    for image in items(&catalog, "services")
        .filter(|image| image["image"]["platform"] == catalog["platform"])
    {
        let reference = reference(image);
        if release(&reference).is_ok() {
            println!("{reference}");
        }
    }
    Ok(())
}

fn render_image(value: &Value) {
    let release = terminal::safe_text(&reference(value));
    let state = match (value["archived"].as_bool(), value["loaded"].as_bool()) {
        (Some(true), _) if value["integrity"] != "verified" => "Archive needs verification",
        (Some(true), Some(true)) => "Verified archive · loaded",
        (Some(true), _) => "Verified archive",
        (_, Some(true)) => "Loaded · archive missing",
        _ => "Archive missing",
    };
    println!("  {release}  ·  {state}");
    if let Some(detail) = value["detail"].as_str() {
        println!("    {}", terminal::safe_text(detail));
    } else if value["status"] == "needs_archive" {
        println!(
            "    Prepare online with devline infra prepare {release}, or import a vault bundle."
        );
    } else if value["status"] == "needs_verification" {
        println!("    Check the archive with devline infra verify {release}.");
    }
}

pub(super) fn render_project_config(value: &Value) {
    if let Some(error) = value["error"].as_str() {
        println!(
            "\nInfrastructure configuration: {}",
            terminal::safe_text(error)
        );
    } else if value["configured"] == true {
        println!("\nInfrastructure");
        for service in items(value, "services") {
            println!("  {}", terminal::safe_text(&reference(service)));
        }
        if let Some(path) = value["path"].as_str() {
            println!(
                "  Check services with devline infra status --project {}.",
                shell_quote(path)
            );
        }
    }
}

fn render_stack(value: &Value) {
    let path = s(value, "path");
    println!("Project  {}", terminal::safe_text(path));
    if value["configured"] != true {
        println!("No infrastructure is configured. See devline infra configure --help.");
        return;
    }
    let state = match (value["running"].as_bool(), value["healthy"].as_bool()) {
        (Some(true), Some(true)) => "Running · healthy",
        (Some(true), _) => "Running · not yet healthy",
        (Some(false), _) if value["docker"]["available"] == false => "Docker unavailable",
        (Some(false), _) => "Stopped",
        (None, _) => "Configured",
    };
    println!(
        "{}  ·  {state}",
        terminal::safe_text(s(value, "project_name"))
    );
    for service in items(value, "services") {
        println!("  {}", terminal::safe_text(&reference(service)));
        for port in items(service, "ports") {
            println!(
                "    {}  127.0.0.1:{}",
                terminal::safe_text(s(port, "name")),
                port["host_port"]
            );
        }
    }
    if let Some(detail) = value["docker"]["detail"].as_str() {
        println!("Docker  {}", terminal::safe_text(detail));
    }
}

pub(super) fn render(action: &str, value: &Value) {
    match action {
        "infra-clients" => {
            println!("Optional {} service clients", s(value, "stack"));
            for service in items(value, "services") {
                for library in items(service, "libraries") {
                    let state = if library["installed"] == true {
                        "Already included"
                    } else if library["offline_available"] == true {
                        "Available locally"
                    } else {
                        "Needs preparation while connected"
                    };
                    println!(
                        "  {}  ·  {}  ·  {state}",
                        terminal::safe_text(s(library, "id")),
                        terminal::safe_text(s(library, "name"))
                    );
                    println!("    {}", terminal::safe_text(s(library, "detail")));
                }
            }
            println!("Select explicitly with devline infra add PATH --client STACK:SERVICE.");
        }
        "infra-clients-added" => {
            println!(
                "{}",
                if value["changed"] == true {
                    "Selected client libraries added and verified offline."
                } else {
                    "Selected client libraries are already included. No dependencies changed."
                }
            );
        }
        "infra-updates" => {
            println!("Image updates");
            if value["checked_at"].is_null() {
                println!("Run devline infra updates check while connected.");
            }
            for item in items(value, "items") {
                let current = terminal::safe_text(s(item, "current_version"));
                let available = item["available_version"].as_str().map(terminal::safe_text);
                let versions = available
                    .filter(|available| *available != current)
                    .map_or_else(
                        || current.clone(),
                        |available| format!("{current} → {available}"),
                    );
                println!(
                    "  {}  {versions}  ·  {}",
                    terminal::safe_text(s(item, "name")),
                    terminal::safe_text(s(item, "state")).replace('_', " ")
                );
                println!("    {}", terminal::safe_text(s(item, "detail")));
                if let Some(reference) = item["reference"].as_str() {
                    println!(
                        "    devline infra update {}",
                        terminal::safe_text(reference)
                    );
                }
            }
        }
        "infra-update-batch" => {
            if items(value, "items").next().is_none() {
                println!("No checked image updates are available.");
            }
            for item in items(value, "items") {
                if let Some(error) = item["error"].as_str() {
                    println!(
                        "  {}  ·  {}",
                        terminal::safe_text(s(item, "reference")),
                        terminal::safe_text(error)
                    );
                } else if value["background"] == true {
                    crate::render_job(&item["result"]);
                } else {
                    render_image(&item["result"]);
                }
            }
        }
        "infra-catalog" => {
            println!(
                "Infrastructure catalog  {}",
                terminal::safe_text(s(value, "platform"))
            );
            for image in items(value, "services").filter(|image| {
                image["image"]["platform"].is_null()
                    || image["image"]["platform"] == value["platform"]
            }) {
                println!(
                    "  {:<26} {}",
                    terminal::safe_text(&reference(image)),
                    terminal::safe_text(s(image, "name"))
                );
            }
            println!("\nPrepare an exact release with devline infra prepare SERVICE@VERSION.");
        }
        "infra-status" => {
            let docker = &value["docker"];
            println!(
                "Local Docker  {}",
                if docker["available"] == true {
                    "Available"
                } else {
                    "Unavailable"
                }
            );
            if let Some(detail) = docker["detail"].as_str() {
                println!("  {}", terminal::safe_text(detail));
            }
            if items(value, "images").next().is_none() {
                println!("No matching images.");
            }
            for image in items(value, "images") {
                render_image(image);
            }
        }
        "infra-image" => render_image(value),
        "infra-removed" => {
            let release = terminal::safe_text(&reference(value));
            if value["removed"] == true {
                println!("Removed the local archive for {release}.");
            } else {
                println!("No local archive for {release}.");
            }
        }
        "infra-exported" => println!(
            "Exported {} image archives to {} ({} bytes).",
            value["images"],
            terminal::safe_text(s(value, "path")),
            value["size_bytes"]
        ),
        "infra-imported" => {
            println!(
                "Imported {} image archives.",
                items(value, "images").count()
            );
            for image in items(value, "images") {
                println!("  {}", terminal::safe_text(&reference(image)));
            }
        }
        "infra-import-inspection" => {
            println!("Image vault preview");
            if let Some(digest) = value["manifest_sha256"].as_str() {
                println!("Manifest SHA-256  {}", terminal::safe_text(digest));
            }
            for image in items(value, "images") {
                println!(
                    "  {}  ·  {}  ·  {} bytes",
                    terminal::safe_text(&reference(image)),
                    terminal::safe_text(s(image, "platform")),
                    image["size_bytes"]
                );
                println!("    {}", terminal::safe_text(s(image, "source")));
                if let Some(detail) = image["detail"].as_str() {
                    println!("    {}", terminal::safe_text(detail));
                }
            }
            if value["requires_trust"] == true {
                println!("This bundle includes publisher pins new to this vault. Review its source and rerun with --trust-images to import.");
            }
        }
        "infra-logs" => print!("{}", terminal::safe_text(s(value, "log"))),
        "infra-credentials" => {
            if let Some(credentials) = value["credentials"].as_object() {
                for (name, secret) in credentials {
                    println!(
                        "{}={}",
                        terminal::safe_text(name),
                        shell_quote(secret.as_str().unwrap_or_default())
                    );
                }
            }
        }
        "infra-env" => {
            if let Some(environment) = value["environment"].as_object() {
                for (name, value) in environment {
                    println!(
                        "export {}={}",
                        name,
                        shell_quote(value.as_str().unwrap_or_default())
                    );
                }
            }
        }
        "infra-stack" | "infra-configured" | "infra-running" | "infra-stopped" => {
            render_stack(value)
        }
        "toolbox" => {
            if let Some(path) = value["path"].as_str() {
                println!("Toolbox  {}", terminal::safe_text(path));
            } else {
                println!("Toolbox");
            }
            println!("{}", match s(value, "status") {
                "ready_offline" => "Ready offline",
                "no_selection" => "No project components selected. Add --doc, --starter or --image, or use --path for a project.",
                _ => "Needs preparation",
            });
            if value["library"]["status"] != "no_selection" {
                println!("\nDocumentation, starters and toolchains");
                render_offline(&value["library"]);
            }
            if let Some(detail) = value["project_dependencies"]["detail"].as_str() {
                println!("\nProject dependencies\n  {}", terminal::safe_text(detail));
            }
            if items(value, "steps").next().is_some() {
                println!("\nPreparation");
                for step in items(value, "steps") {
                    println!(
                        "  {} · {} · {}",
                        terminal::safe_text(s(step, "kind")),
                        terminal::safe_text(s(step, "id")),
                        terminal::safe_text(s(step, "status"))
                    );
                    if let Some(detail) = step["detail"].as_str() {
                        println!("    {}", terminal::safe_text(detail));
                    }
                }
            }
            if items(value, "blockers").next().is_some() {
                println!("\nStill needed");
                for blocker in items(value, "blockers") {
                    println!(
                        "  {}: {}",
                        terminal::safe_text(s(blocker, "id")),
                        terminal::safe_text(s(blocker, "detail"))
                    );
                }
                if let Some(path) = value["path"].as_str() {
                    println!(
                        "  Prepare together: devline toolbox prepare --path {}",
                        shell_quote(path)
                    );
                }
            }
            if items(value, "images").next().is_some() {
                println!("\nImage vault");
                for image in items(value, "images") {
                    render_image(image);
                }
                if value["docker"]["available"] == false {
                    println!("  Start a local Docker daemon to run the selected services.");
                    if let Some(detail) = value["docker"]["detail"].as_str() {
                        println!("  {}", terminal::safe_text(detail));
                    }
                }
            }
        }
        _ => unreachable!("unexpected infrastructure output"),
    }
}
