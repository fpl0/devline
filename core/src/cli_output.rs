//! Terminal presentation for standard command results.

use crate::{cli_infra, s, terminal, terminal::shell_quote, Cli};
use anyhow::{Context, Result};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn render_packages(value: &Value, empty: &str) {
    if let Some(items) = value.as_array() {
        if items.is_empty() {
            println!("{empty}");
        }
        for item in items {
            println!(
                "{:<30} {:<12} {:<10} {}",
                format!("{}@{}", s(item, "id"), s(item, "version")),
                s(item, "stack"),
                if item["installed"] == true {
                    "installed"
                } else {
                    "available"
                },
                s(item, "name")
            );
        }
    }
}
fn render_shelves(value: &Value) {
    // One runtime prints as a package list; several print as a summary of each shelf.
    let single = [value.clone()];
    let stacks = value["stacks"]
        .as_array()
        .map_or(&single[..], Vec::as_slice);
    let detailed = stacks.len() == 1;
    for stack in stacks {
        let shelf = &stack["shelf"];
        let size = shelf["bytes"]
            .as_u64()
            .map(|bytes| format!(" · {:.0} MB", bytes as f64 / 1_000_000.0))
            .unwrap_or_default();
        println!(
            "{:<8} {:<14} {}",
            s(stack, "stack"),
            s(stack, "registry"),
            if shelf["ready"] == true {
                format!(
                    "{} of {} packages on this Mac{size}",
                    shelf["count"], shelf["total"]
                )
            } else {
                format!("not downloaded · {} packages available", shelf["total"])
            }
        );
        if !detailed {
            continue;
        }
        for package in stack["packages"].as_array().into_iter().flatten() {
            let state = if package["installed"] == true {
                "in project".to_owned()
            } else if package["available"] == true {
                s(package, "version").to_owned()
            } else if package["error"].is_string() {
                "unavailable".to_owned()
            } else {
                "—".to_owned()
            };
            println!(
                "  {:<38} {:<14} {}",
                terminal::safe_text(s(package, "name")),
                terminal::safe_text(&state),
                terminal::safe_text(s(package, "summary"))
            );
        }
        if shelf["ready"] != true {
            println!(
                "Download while connected with devline packages download {}.",
                s(stack, "stack")
            );
        }
    }
    if stacks.is_empty() {
        println!("No package shelves are defined in this catalog.");
    }
}

fn render_runtimes(value: &Value) -> Result<()> {
    let items = value.as_array().context("Unexpected runtime list")?;
    if items.is_empty() {
        println!("No toolchain checks are available in this catalog.");
    }
    for item in items {
        let available = item["available"] == true;
        println!(
            "{} [{}] · {}",
            s(item, "name"),
            s(item, "id"),
            if available {
                "Available"
            } else {
                "Needs setup"
            }
        );
        println!(
            "  Starter baseline  {} {}",
            s(item, "toolchain"),
            s(item, "minimum_version")
        );
        for line in s(item, "detail").lines() {
            println!("  {line}");
        }
        if !available {
            let guide = &item["install"];
            for command in guide["commands"].as_array().into_iter().flatten() {
                println!(
                    "  Install with {:<9} {}",
                    s(command, "manager"),
                    s(command, "command")
                );
            }
            if !s(guide, "url").is_empty() {
                println!("  Or download         {}", s(guide, "url"));
            }
            if !s(guide, "note").is_empty() {
                println!("  {}", s(guide, "note"));
            }
        }
        for (field, label) in [("docs", "Docs"), ("templates", "Starters")] {
            if let Some(entries) = item[field].as_array() {
                let ids: Vec<_> = entries.iter().filter_map(Value::as_str).collect();
                if !ids.is_empty() {
                    println!("  {label}  {}", ids.join(", "));
                }
            }
        }
        println!();
    }
    Ok(())
}
fn update_command_prefix(cli: &Cli) -> String {
    let mut command = String::from("devline");
    for (flag, path) in [("--home", &cli.home), ("--catalog", &cli.catalog)] {
        if let Some(path) = path {
            command.push_str(&format!(" {flag} {}", shell_quote(&path.to_string_lossy())));
        }
    }
    command
}

fn last_update_check(checked_at: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(checked_at, |duration| duration.as_secs());
    let elapsed = now.saturating_sub(checked_at);
    let (amount, unit) = match elapsed {
        0..=59 => return "just now".to_owned(),
        60..=3599 => (elapsed / 60, "minute"),
        3600..=86399 => (elapsed / 3600, "hour"),
        _ => (elapsed / 86400, "day"),
    };
    format!("{amount} {unit}{} ago", if amount == 1 { "" } else { "s" })
}

fn render_update_action(item: &Value, command: &str) {
    let reference = s(item, "reference");
    match s(item, "action") {
        "install_docs" if !reference.is_empty() => {
            let (id, version) = reference
                .split_once('@')
                .unwrap_or((reference, s(item, "available_version")));
            if !id.is_empty() && !version.is_empty() {
                if item["delivery"] == "local" {
                    println!(
                        "    Install      {command} docs install {} --version {}",
                        shell_quote(id),
                        shell_quote(version)
                    );
                } else {
                    println!(
                        "    Download     {command} docs fetch {}",
                        shell_quote(&format!("{id}@{version}"))
                    );
                }
            }
        }
        "rebuild_template" if !reference.is_empty() => println!(
            "    Build online {command} templates build {} --refresh-deps",
            shell_quote(reference)
        ),
        "install_template" if !reference.is_empty() => println!(
            "    Prepare      {command} updates apply {}",
            shell_quote(reference)
        ),
        "update_image" if !reference.is_empty() => println!(
            "    Archive      {command} infra update {}",
            shell_quote(reference)
        ),
        "prepare_local_docs" => {
            println!("    Prepare      {command} --offline docs install swiftui")
        }
        "open_url" => {
            let url = s(item, "url");
            if url.starts_with("https://") {
                println!("    Open         {url}");
            }
        }
        _ if item["kind"] == "app" && item["state"] == "unconfigured" => {
            println!("    Configure    {command} updates configure <manifest-url>");
        }
        _ => (),
    }
}

fn render_updates(cli: &Cli, snapshot: &Value) -> Result<()> {
    let items = snapshot["items"]
        .as_array()
        .context("Unexpected update status")?;
    let command = update_command_prefix(cli);
    if let Some(checked_at) = snapshot["checked_at"].as_u64() {
        println!(
            "Update status · last checked {}",
            last_update_check(checked_at)
        );
    } else {
        println!("Update status · no online check recorded");
        println!("Run {command} updates check when you’re ready to connect.");
    }
    for source in snapshot["sources"].as_array().into_iter().flatten() {
        if matches!(source["state"].as_str(), Some("error" | "unchecked")) {
            println!(
                "  {} · {}",
                terminal::safe_text(s(source, "name")),
                terminal::safe_text(s(source, "detail"))
            );
        }
    }
    let available = items
        .iter()
        .filter(|item| item["state"] == "available")
        .count();
    let failed = items.iter().filter(|item| item["state"] == "error").count();
    if available > 0 || failed > 0 {
        println!("{available} available · {failed} source checks failed");
    }

    for (kind, title) in [
        ("docs", "Documentation"),
        ("template", "Templates"),
        ("image", "Infrastructure Images"),
        ("runtime", "Runtimes"),
        ("app", "Devline App"),
    ] {
        let mut group: Vec<_> = items.iter().filter(|item| item["kind"] == kind).collect();
        if group.is_empty() {
            continue;
        }
        group.sort_by_key(|item| match s(item, "state") {
            "available" => 0,
            "error" => 1,
            "unchecked" => 2,
            _ => 3,
        });
        println!("\n{title}");
        for item in group {
            let versions = match (s(item, "current_version"), s(item, "available_version")) {
                ("", "") => String::new(),
                ("", available) => format!("  available {available}"),
                (current, available) if !available.is_empty() && current != available => {
                    format!("  {current} → {available}")
                }
                (current, _) => format!("  {current}"),
            };
            let status = match s(item, "state") {
                "current" => "Up to date",
                "available" => "Update available",
                "unchecked" => "Unchecked",
                "error" => "Check failed",
                "unconfigured" => "Local build",
                _ => "Unknown status",
            };
            println!("  {}{versions} · {status}", s(item, "name"));
            for line in s(item, "detail").lines() {
                println!("    {line}");
            }
            render_update_action(item, &command);
        }
    }
    Ok(())
}
fn readiness(value: &Value) {
    let template = &value["template"];
    let release = format!("{}@{}", s(template, "id"), s(template, "version"));
    println!("{} · {}", release, s(template, "target"));
    println!(
        "{}\n",
        if value["ready"] == true {
            "Ready to create and test offline on this Mac."
        } else {
            "Preparation needed before offline use."
        }
    );
    if let Some(checks) = value["checks"].as_array() {
        for check in checks {
            println!(
                "  {:<8} {} — {}",
                if check["ok"] == true {
                    "ready"
                } else {
                    "missing"
                },
                s(check, "name"),
                s(check, "detail")
            );
        }
        if checks
            .iter()
            .any(|check| check["name"] == "Installed release" && check["ok"] == false)
        {
            println!("\nInstall the starter and matching docs with devline prepare {release}");
        }
    }
}

fn render_project(value: &Value) {
    println!("Project  {}", terminal::safe_text(s(value, "path")));
    if value["stacks"].as_array().is_none_or(Vec::is_empty) {
        println!("No supported project manifests found.");
    }
    for dependency in value["dependencies"].as_array().into_iter().flatten() {
        let version = dependency["version"]
            .as_str()
            .or(dependency["requirement"].as_str())
            .unwrap_or("version unspecified");
        println!(
            "  {:<14} {}  ·  {}",
            s(dependency, "id"),
            terminal::safe_text(version),
            terminal::safe_text(s(dependency, "source"))
        );
    }
    if let Some(docsets) = value["docsets"]
        .as_array()
        .filter(|packs| !packs.is_empty())
    {
        println!("\nDocumentation");
        for pack in docsets {
            let reference = pack["version"].as_str().map_or_else(
                || s(pack, "id").to_owned(),
                |version| format!("{}@{version}", s(pack, "id")),
            );
            println!(
                "  {reference:<28} {}",
                terminal::safe_text(s(pack, "detail"))
            );
        }
    }
    cli_infra::render_project_config(&value["infrastructure"]);
}

pub(super) fn render_job(job: &Value) {
    println!(
        "{}  {}  {}",
        s(job, "id"),
        s(job, "kind").replace('_', " "),
        s(job, "state")
    );
    if let Some(stage) = job["stage"].as_str() {
        println!("  {}", terminal::safe_text(stage));
    }
    if let Some(error) = job["error"].as_str() {
        println!("  {}", terminal::safe_text(error));
    }
    if job["cancel_requested"] == true
        && matches!(job["state"].as_str(), Some("queued" | "running"))
    {
        println!("  Cancellation requested.");
    }
    if let (Some(done), Some(total)) =
        (job["completed_bytes"].as_u64(), job["total_bytes"].as_u64())
    {
        println!("  {done} / {total} bytes");
    }
}

fn render_doc_updates(value: &Value) {
    if let Some(channel) = value["channel_url"].as_str() {
        println!("Documentation channel  {channel}");
    } else {
        println!("No documentation release channel configured.");
    }
    if let Some(error) = value["error"].as_str() {
        println!("Last check  {}", terminal::safe_text(error));
    }
    for pack in value["packs"].as_array().into_iter().flatten() {
        let active = pack["active_version"].as_str().unwrap_or("not installed");
        let latest = pack["latest"]["pack"]["version"]
            .as_str()
            .unwrap_or("no channel release");
        println!(
            "  {:<18} {:<12} {active}  ·  {latest}",
            s(pack, "id"),
            s(pack, "state")
        );
    }
}

pub(super) fn render_offline(value: &Value) {
    println!(
        "Offline readiness  {}",
        if value["ready_offline"] == true {
            "Ready"
        } else {
            "Needs attention"
        }
    );
    for pack in value["docs"].as_array().into_iter().flatten() {
        let state = if pack["ready_offline"] == true {
            "ready"
        } else {
            "missing"
        };
        println!(
            "  {state:<8} {}@{}  {}",
            s(pack, "id"),
            s(pack, "version"),
            terminal::safe_text(s(pack, "detail"))
        );
    }
    for starter in value["starters"].as_array().into_iter().flatten() {
        if let Some(error) = starter["error"].as_str() {
            println!(
                "  {}  {}",
                terminal::safe_text(s(starter, "id")),
                terminal::safe_text(error)
            );
        } else {
            readiness(starter);
        }
    }
    for runtime in value["runtimes"].as_array().into_iter().flatten() {
        println!(
            "  {}  {}",
            terminal::safe_text(s(runtime, "name")),
            terminal::safe_text(s(runtime, "detail"))
        );
    }
}

fn render_bundle(value: &Value) {
    println!(
        "Bundle  {}",
        if value["compatible"] == true {
            "Compatible"
        } else {
            "Needs attention"
        }
    );
    println!(
        "{} files · {} bytes",
        value["file_count"], value["size_bytes"]
    );
    for item in value["docs"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(value["starters"].as_array().into_iter().flatten())
    {
        println!("  {}@{}", s(item, "id"), s(item, "version"));
    }
    for problem in value["problems"].as_array().into_iter().flatten() {
        println!(
            "  {}  {}",
            s(problem, "id"),
            terminal::safe_text(s(problem, "reason"))
        );
    }
    if value["requires_trust"] == true {
        println!("Installing executable starters requires --trust-starters.");
    }
}

fn render_starter_details(value: &Value) {
    let starter = &value["template"];
    println!("{}@{}", s(starter, "id"), s(starter, "version"));
    println!(
        "{} files · {} bytes",
        value["file_count"], value["size_bytes"]
    );
    println!(
        "Offline readiness  {}",
        if value["ready_offline"] == true {
            "Ready"
        } else {
            "Needs attention"
        }
    );
    if let Some(digest) = value["digest"].as_str() {
        println!("SHA-256  {digest}");
    }
    if let Some(contents) = value["contents"]
        .as_array()
        .filter(|items| !items.is_empty())
    {
        println!("\nProject contents");
        for item in contents {
            let suffix = if item["kind"] == "directory" { "/" } else { "" };
            println!("  {}{suffix}", terminal::safe_text(s(item, "name")));
        }
    }
    if let Some(dependencies) = value["dependencies"]
        .as_array()
        .filter(|items| !items.is_empty())
    {
        println!("\nDependencies");
        for dep in dependencies {
            println!(
                "  {}  {}",
                terminal::safe_text(s(dep, "name")),
                terminal::safe_text(s(dep, "version"))
            );
        }
    }
}

pub(super) fn render_reading_list(data: &Value, saved: bool, pick: bool) -> Result<()> {
    let items = data.as_array().context("Unexpected reading list")?;
    if items.is_empty() {
        println!(
            "{}",
            if saved {
                "No saved references. Save a symbol or exact article ID with devline docs save."
            } else {
                "No recent references. Open an article in Devline or with devline docs show."
            }
        );
        return Ok(());
    }
    for (index, item) in items.iter().enumerate() {
        if pick {
            print!("{:>2}. ", index + 1);
        }
        let unavailable = if item["installed"] == false {
            " · Pack removed"
        } else {
            ""
        };
        println!(
            "{}  [{}@{}{unavailable}]\n  {}",
            terminal::safe_text(s(item, "title")),
            s(item, "pack_id"),
            s(item, "version"),
            s(item, "id")
        );
        if let Some(focus) = item["focus_text"].as_str() {
            println!("  {}", terminal::safe_text(focus).trim());
        }
    }
    Ok(())
}

pub(super) fn render(cli: &Cli, action: &str, data: &Value) -> Result<()> {
    match action {
        "local-docs" => {
            println!("SwiftUI · Official API documentation from local Xcode");
            if let Some(version) = data["sdk_version"].as_str() {
                println!("macOS SDK {version}");
            }
            println!("{}", terminal::safe_text(s(data, "detail")));
            if let Some(count) = data["count"].as_u64() {
                println!("{count} symbols indexed for this SDK.");
            }
            println!(
                "\nPrepare or refresh: {} --offline docs install swiftui --background",
                update_command_prefix(cli)
            );
            println!("This reference stays on this Mac and is excluded from exports.");
        }
        "project" => render_project(data),
        "guides" => {
            println!("Devline Help");
            for guide in data.as_array().context("Unexpected guide list")? {
                println!(
                    "  {}  {}",
                    s(guide, "id"),
                    terminal::safe_text(s(guide, "title"))
                );
            }
        }
        "offline" => render_offline(data),
        action if action == "toolbox" || action.starts_with("infra-") => {
            cli_infra::render(action, data)
        }
        "verified" => {
            let template = &data["template"];
            println!(
                "Verified {}@{} with external networking denied.",
                s(template, "id"),
                s(template, "version")
            );
        }
        "bundle" => render_bundle(data),
        "bundle-exported" => println!(
            "Exported {} docsets and {} starters to {}.",
            data["docs"],
            data["starters"],
            s(data, "path")
        ),
        "bundle-imported" => println!(
            "Imported {} docsets and {} starters.",
            data["docs"].as_array().map_or(0, Vec::len),
            data["starters"].as_array().map_or(0, Vec::len)
        ),
        "jobs" => {
            let jobs = data.as_array().context("Unexpected job list")?;
            if jobs.is_empty() {
                println!("No background jobs.");
            }
            for job in jobs {
                render_job(job);
            }
        }
        "job" => {
            render_job(data);
            if let Some(log) = data["log"].as_str() {
                print!("{}", terminal::safe_text(log));
            }
        }
        "job-started" => {
            render_job(data);
            println!(
                "Follow with devline jobs status {} --follow.",
                s(data, "id")
            );
        }
        "doc-updates" => render_doc_updates(data),
        "doc-fetched" => println!(
            "Installed and activated {}@{} · {} articles.",
            s(data, "id"),
            s(data, "version"),
            data["count"]
        ),
        "doc-rollback" => println!(
            "Active documentation is {}@{}.",
            s(data, "id"),
            s(data, "active_version")
        ),
        "template-detail" => render_starter_details(data),
        "template-removed" => println!("Removed the installed starter release."),
        "runtime-list" => render_runtimes(data)?,
        "packages" | "packages-downloaded" => render_shelves(data),
        "packages-added" => println!(
            "{}",
            match s(data, "status") {
                "already_included" => "The project already uses these packages.",
                "unchanged" => "Nothing to add.",
                _ => "Packages added. The project passed its checks with the network denied.",
            }
        ),
        "packages-removed" => println!(
            "{}",
            if data["removed"] == true { "Shelf removed." } else { "That shelf was not downloaded." }
        ),
        "updates" => render_updates(cli, data)?,
        "content-updated" => {
            println!(
                "{}",
                if data["complete"] == true {
                    "Content updates complete"
                } else {
                    "Content updates need attention"
                }
            );
            for step in data["steps"].as_array().into_iter().flatten() {
                println!(
                    "  {} · {}",
                    terminal::safe_text(s(step, "name")),
                    terminal::safe_text(s(step, "state"))
                );
                if let Some(detail) = step["detail"].as_str() {
                    println!("    {}", terminal::safe_text(detail));
                }
            }
        }
        "updates-configured" => println!(
            "App update channel saved.\nRun {} updates check when you’re ready to connect.",
            update_command_prefix(cli)
        ),
        "updates-cleared" => {
            println!("App update channel cleared. Devline will show Local build.")
        }
        "library-retention" => println!(
            "Keep only the latest local version after updates: {}",
            if data["keep_latest"] == true { "on" } else { "off" }
        ),
        "library-purged" => println!(
            "Older releases removed\n  Documentation  {}\n  App starters   {}\n  Image archives  {}",
            data["docs"], data["starters"], data["images"]
        ),
        "docs-list" => render_packages(
            data,
            "No matching docs packs. Run devline docs list to see the full catalog.",
        ),
        "saved" => render_reading_list(data, true, false)?,
        "recent" => render_reading_list(data, false, false)?,
        "saved-article" => {
            println!("Saved reference. Find it in Devline or with devline docs saved.")
        }
        "unsaved-article" => println!("Saved reference removed."),
        "template-list" => render_packages(
            data,
            "No matching starter releases. Run devline templates list to browse all releases.",
        ),
        "catalog" => {
            println!("Documentation\n");
            render_packages(&data["docs"], "No docs packs available.");
            println!("\nTemplates\n");
            render_packages(&data["templates"], "No starter releases available.");
            println!("\nLibrary  {}", s(data, "home"));
        }
        "doctor" => readiness(data),
        "prepare" => {
            let template = &data["template"];
            println!(
                "Installed starter {}@{}",
                s(template, "id"),
                s(template, "version")
            );
            if let Some(docs) = data["docs"].as_array() {
                for pack in docs {
                    println!(
                        "Installed docs    {}@{} · {} articles",
                        s(pack, "id"),
                        s(pack, "version"),
                        pack["count"]
                    );
                }
            }
            println!();
            readiness(&data["doctor"]);
        }
        "new" => {
            let dry_run = data["dry_run"] == true;
            let release = format!("{}@{}", s(data, "template"), s(data, "version"));
            let path = s(data, "path");
            if dry_run {
                println!(
                    "Project plan  {}\nTemplate  {release}",
                    terminal::safe_text(path)
                );
                let services = &data["service_plan"];
                if !services.is_null() {
                    println!("\nProject services");
                    for image in services["images"].as_array().into_iter().flatten() {
                        let operation = if image["archived"] == true {
                            "verify local archive"
                        } else if data["offline"] == true {
                            "archive required before creation"
                        } else {
                            "prepare archive while connected"
                        };
                        println!(
                            "  {}@{}  ·  {operation}",
                            s(image, "id"),
                            s(image, "version")
                        );
                    }
                    if let Some(ports) = services["ports"].as_object() {
                        for (port, host) in ports {
                            println!("  {}  →  127.0.0.1:{host}", terminal::safe_text(port));
                        }
                    }
                    if services["docker"]["available"] != true
                        || services["docker"]["platform"] != services["platform"]
                    {
                        println!("  Start a compatible local Docker engine before creating this project.");
                    }
                }
                println!(
                    "\n{}",
                    if data["can_create"] == true {
                        "Ready to create. Rerun this command without --dry-run."
                    } else {
                        "Preparation needed before creation."
                    }
                );
            } else {
                println!(
                    "Created {path}\nTemplate  {release}\n\nTest offline\n  cd {}\n  {}",
                    shell_quote(path),
                    s(data, "test_command")
                );
                let services = data["services"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(|service| format!(" --service {}", shell_quote(service)))
                    .collect::<String>();
                if data["infrastructure"]["configured"] == true {
                    println!(
                        "\nStart the configured services offline\n  {} --offline infra run {}",
                        update_command_prefix(cli),
                        shell_quote(path)
                    );
                } else if !services.is_empty() {
                    println!(
                        "\nConfigure project services\n  {} infra configure {}{services}",
                        update_command_prefix(cli),
                        shell_quote(path)
                    );
                }
            }
        }
        "install" => println!(
            "Installed {}@{}{}",
            s(data, "id"),
            s(data, "version"),
            data.get("count")
                .map(|c| format!(" · {c} documents"))
                .unwrap_or_default()
        ),
        "remove" => println!("Documentation removed from the local library."),
        "build" => println!(
            "Built and installed {}@{}\nPassed network-denied verification.\n{}",
            s(data, "id"),
            s(data, "version"),
            s(data, "path")
        ),
        _ => println!("{}", serde_json::to_string_pretty(data)?),
    }
    Ok(())
}
