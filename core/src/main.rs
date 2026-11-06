use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Parser)]
#[command(
    name = "devline",
    version,
    about = "Your offline developer library",
    long_about = "Search official docs and create projects from installed starters. The CLI shares its local library with Devline.app and works with the app closed.",
    after_long_help = "Examples:\n  devline library                             Browse bundled docs and starters\n  devline runtimes                            Check toolchains on this Mac\n  devline docs install go                     Add the Go reference to your library\n  devline docs search 'http.Handler' --pick    Search and read in the terminal\n  devline prepare go/chi-api                   Install the starter and matching docs\n  devline new go/chi-api ./status-api          Create a project offline\n  devline updates                             Read saved update status\n  devline updates check                       Check upstream sources online\n\nRun 'devline completions --help' for shell setup. The previous 'devkit' command remains a compatibility alias."
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Emit structured JSON, suitable for scripts"
    )]
    json: bool,
    #[arg(long, global = true, help = "Library storage directory")]
    home: Option<PathBuf>,
    #[arg(long, global = true, help = "Directory containing catalog.json")]
    catalog: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Require local resources (the default for read/create commands)"
    )]
    offline: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    #[command(
        about = "Search and read installed documentation",
        after_long_help = "Run 'devline docs' for interactive search. With a single detected stack in the current project, search defaults to that stack. Use --stack all to search the full installed library."
    )]
    Docs {
        #[command(subcommand)]
        command: Option<Docs>,
    },
    #[command(about = "Browse, install, and build template releases")]
    Templates {
        #[command(subcommand)]
        command: Templates,
    },
    #[command(about = "Inspect local runtimes and toolchains")]
    Runtimes {
        #[command(subcommand)]
        command: Option<Runtimes>,
    },
    #[command(
        about = "Read saved update status or explicitly check online",
        after_long_help = "Examples:\n  devline updates\n  devline updates check --json\n  devline updates configure https://downloads.example.com/devline-release.json\n  devline updates configure --clear"
    )]
    Updates {
        #[command(subcommand)]
        command: Option<Updates>,
    },
    #[command(about = "Install a starter and matching docs, then check offline readiness")]
    Prepare {
        /// Starter ID or exact release, for example go/chi-api@1.0.0.
        id: String,
    },
    #[command(about = "Check a starter and its toolchain on this Mac")]
    Doctor { id: String },
    #[command(
        about = "Create a project from an installed starter",
        after_long_help = "Examples:\n  devline new go/chi-api ./status-api\n  devline new go/chi-api@1.0.0 ./status-api --dry-run\n\nWithout arguments, Devline prompts for an installed starter and destination."
    )]
    New {
        /// Starter ID or exact release.
        id: Option<String>,
        /// Destination directory for the generated project.
        destination: Option<PathBuf>,
        #[arg(long)]
        /// Check the selected release and destination before creating files.
        dry_run: bool,
    },
    #[command(about = "List available packs and installed versions")]
    Library,
    #[command(
        about = "Print shell completions",
        after_long_help = "Install zsh completions with live local pack and starter IDs:\n  mkdir -p ~/.zfunc\n  devline completions zsh > ~/.zfunc/_devline\n  # Add the following to ~/.zshrc, before any existing compinit call:\n  fpath=(~/.zfunc $fpath)\n  autoload -Uz compinit; compinit\n\nLoad for the current Bash session:\n  source <(devline completions bash)\n\nInstall for Fish:\n  mkdir -p ~/.config/fish/completions\n  devline completions fish > ~/.config/fish/completions/devline.fish"
    )]
    Completions { shell: clap_complete::Shell },
}
#[derive(Subcommand)]
enum Docs {
    #[command(about = "Find symbols and articles in installed packs")]
    Search {
        query: String,
        #[arg(long)]
        /// Limit results to a stack such as go or dotnet, or use all.
        stack: Option<String>,
        #[arg(long)]
        /// Match an exact documentation release.
        version: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        /// Choose a result and read it in the terminal.
        pick: bool,
    },
    #[command(about = "Read an article by symbol or exact result ID")]
    Show {
        id: String,
        #[arg(long)]
        stack: Option<String>,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        /// Print the article directly to stdout.
        no_pager: bool,
        #[arg(long)]
        /// Open the article in Devline.app.
        open: bool,
    },
    #[command(about = "Add a documentation pack from the bundled catalog")]
    Install {
        id: String,
        #[arg(long)]
        version: Option<String>,
    },
    #[command(about = "List available documentation packs")]
    List {
        #[arg(long)]
        installed: bool,
    },
    #[command(about = "Remove an installed documentation release")]
    Remove {
        id: String,
        #[arg(long)]
        version: String,
    },
}
#[derive(Subcommand)]
enum Templates {
    #[command(about = "List available starter releases")]
    List {
        #[arg(long)]
        installed: bool,
        #[arg(long = "type")]
        app_type: Option<String>,
        #[arg(long)]
        stack: Option<String>,
    },
    #[command(about = "Install a starter release from the bundled catalog")]
    Install { id: String },
    #[command(about = "Explicitly rebuild a recipe using current upstream dependencies")]
    Build {
        id: String,
        #[arg(long, required = true)]
        refresh_deps: bool,
    },
}
#[derive(Subcommand)]
enum Runtimes {
    #[command(about = "List supported stacks and report toolchain availability on this Mac")]
    List,
}
#[derive(Subcommand)]
enum Updates {
    #[command(about = "Read saved update status from the local library")]
    Status,
    #[command(about = "Check upstream sources online and save the result")]
    Check,
    #[command(about = "Set or clear the HTTPS manifest used for Devline app releases")]
    Configure {
        #[arg(required_unless_present = "clear", conflicts_with = "clear", value_hint = clap::ValueHint::Url)]
        /// Public HTTPS release-manifest URL.
        url: Option<String>,
        #[arg(long)]
        /// Remove the app update channel and return to local-build status.
        clear: bool,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum CompletionKind {
    AvailableDocs,
    InstalledDocs,
    AvailableStarters,
    InstalledStarters,
}
fn answer(label: &str) -> Result<Option<String>> {
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        bail!("Arguments are required in scripts. See devline --help.");
    }
    eprint!("{label} ");
    io::stderr().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    let value = value.trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}
fn ask(label: &str) -> Result<String> {
    answer(label)?.context("Cancelled")
}
fn progress(cli: &Cli, message: std::fmt::Arguments<'_>) {
    if !cli.json && io::stderr().is_terminal() {
        eprintln!("{message}");
    }
}
fn request(cli: &Cli, mut request: Value) -> Result<Value> {
    if let Some(home) = &cli.home {
        request["home"] = json!(home);
    }
    if let Some(catalog) = &cli.catalog {
        request["catalog"] = json!(catalog);
    }
    let all_stacks = request["stack"].as_str() == Some("all");
    if all_stacks {
        request["stack"] = Value::Null;
    } else if matches!(
        request["command"].as_str(),
        Some("docs.search" | "docs.show")
    ) && request["stack"].is_null()
    {
        let exact_result = request["command"] == "docs.show"
            && request["id"]
                .as_str()
                .is_some_and(|id| id.contains('@') && id.contains('/'));
        if !exact_result {
            if let Ok(cwd) = std::env::current_dir() {
                request["stack"] = devkit_core::project_context(&cwd)["stack"].clone();
            }
        }
    }
    devkit_core::execute(&request)
}

fn completion_values(cli: &Cli, kind: CompletionKind) -> Result<()> {
    let (docs, installed) = match kind {
        CompletionKind::AvailableDocs => (true, false),
        CompletionKind::InstalledDocs => (true, true),
        CompletionKind::AvailableStarters => (false, false),
        CompletionKind::InstalledStarters => (false, true),
    };
    let data = if docs {
        request(cli, json!({"command":"catalog"}))?["docs"].clone()
    } else {
        request(cli, json!({"command":"templates.list"}))?
    };
    let mut choices = BTreeSet::new();
    for item in data
        .as_array()
        .context("Unexpected completion candidates")?
    {
        if installed && item["installed"] != true {
            continue;
        }
        let id = s(item, "id");
        if id.is_empty()
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_/".contains(&b))
        {
            continue;
        }
        choices.insert(id.to_owned());
        let version = s(item, "version");
        if !docs && !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        {
            choices.insert(format!("{id}@{version}"));
        }
    }
    for choice in choices {
        println!("{choice}");
    }
    Ok(())
}

fn completion_argument(command: clap::Command, marker: &'static str) -> clap::Command {
    command.mut_arg("id", |argument| argument.value_parser([marker]))
}

fn print_completions(shell: clap_complete::Shell) -> Result<()> {
    let mut command = Cli::command();
    if shell != clap_complete::Shell::Zsh {
        clap_complete::generate(shell, &mut command, "devline", &mut io::stdout());
        return Ok(());
    }
    // Mark these positions on the completion command only. Runtime argument parsing
    // still accepts any valid catalog ID, including releases added after this script.
    command = command
        .mut_subcommand("docs", |command| {
            command
                .mut_subcommand("install", |cmd| {
                    completion_argument(cmd, "__available_docs")
                })
                .mut_subcommand("remove", |cmd| completion_argument(cmd, "__installed_docs"))
        })
        .mut_subcommand("templates", |command| {
            command
                .mut_subcommand("install", |cmd| {
                    completion_argument(cmd, "__available_starters")
                })
                .mut_subcommand("build", |cmd| {
                    completion_argument(cmd, "__available_starters")
                })
        })
        .mut_subcommand("doctor", |cmd| {
            completion_argument(cmd, "__available_starters")
        })
        .mut_subcommand("prepare", |cmd| {
            completion_argument(cmd, "__available_starters")
        })
        .mut_subcommand("new", |cmd| {
            completion_argument(cmd, "__installed_starters")
        });
    let mut generated = Vec::new();
    clap_complete::generate(shell, &mut command, "devline", &mut generated);
    let script = String::from_utf8(generated)?
        .replace("(__available_docs)", "_devline_local_ids available-docs")
        .replace("(__installed_docs)", "_devline_local_ids installed-docs")
        .replace(
            "(__available_starters)",
            "_devline_local_ids available-starters",
        )
        .replace(
            "(__installed_starters)",
            "_devline_local_ids installed-starters",
        );
    let (header, body) = script
        .split_once('\n')
        .context("Missing zsh completion header")?;
    println!("{header}");
    print!(
        r#"
_devline_local_ids() {{
    local -a candidates catalog_args
    local kind="${{argv[-1]}}"
    shift -p 1
    [[ -n "${{opt_args[--home]-}}" ]] && catalog_args+=(--home "${{opt_args[--home]}}")
    [[ -n "${{opt_args[--catalog]-}}" ]] && catalog_args+=(--catalog "${{opt_args[--catalog]}}")
    candidates=("${{(@f)$(_DEVKIT_ZSH_COMPLETION="$kind" command devline "${{catalog_args[@]}}" 2>/dev/null)}}")
    [[ -n "${{candidates[1]-}}" ]] && compadd "$@" -a candidates
}}
{body}"#
    );
    Ok(())
}
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
fn s<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("")
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
                println!(
                    "    Install      {command} docs install {} --version {}",
                    shell_quote(id),
                    shell_quote(version)
                );
            }
        }
        "rebuild_template" if !reference.is_empty() => println!(
            "    Build online {command} templates build {} --refresh-deps",
            shell_quote(reference)
        ),
        "install_template" if !reference.is_empty() => println!(
            "    Install      {command} templates install {}",
            shell_quote(reference)
        ),
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
fn page(text: &str, disable: bool) -> Result<()> {
    if !disable && io::stdout().is_terminal() && std::env::var("TERM").unwrap_or_default() != "dumb"
    {
        let mut child = Command::new("/usr/bin/less")
            .arg("-FRX")
            .stdin(Stdio::piped())
            .spawn()
            .context("Could not open pager; use --no-pager")?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        child.wait()?;
    } else {
        println!("{text}");
    }
    Ok(())
}
fn read_article(result: &Value, no_pager: bool) -> Result<()> {
    page(
        &format!(
            "{}\n{} · {}\n{}\n\n{}",
            s(result, "title"),
            s(result, "pack_id"),
            s(result, "version"),
            s(result, "source_url"),
            s(result, "body")
        ),
        no_pager,
    )
}
fn search_docs(
    cli: &Cli,
    query: &str,
    stack: Option<&str>,
    version: Option<&str>,
    limit: u32,
    pick: bool,
) -> Result<i32> {
    if pick && cli.json {
        bail!("Choose --pick for interactive reading or --json for structured results.");
    }
    if pick
        && (!io::stdin().is_terminal()
            || !io::stdout().is_terminal()
            || !io::stderr().is_terminal())
    {
        bail!(
            "--pick needs an interactive terminal. Use devline docs search for pipeable results."
        );
    }
    let data = request(
        cli,
        json!({"command":"docs.search","query":query,"stack":stack,"version":version,"limit":limit}),
    )?;
    let items = data.as_array().context("Unexpected search results")?;
    if cli.json {
        println!("{}", json!({"schema_version":1,"data":data}));
    } else if items.is_empty() {
        println!("No installed documentation matched {query:?}.");
        if stack != Some("all") || version.is_some() {
            println!("Try a broader term or search all installed releases with --stack all and omit --version.");
        }
        println!(
            "Run devline docs list to see available packs and devline docs install <id> to add one."
        );
    } else {
        for (index, item) in items.iter().enumerate() {
            if pick {
                print!("{:>2}. ", index + 1);
            }
            println!(
                "{}  [{} · {}]\n  {}\n  {}\n",
                s(item, "title"),
                s(item, "stack"),
                s(item, "version"),
                s(item, "id"),
                s(item, "snippet").replace('\n', " ")
            );
        }
        if pick {
            while let Some(choice) = answer("Open result number (Return to exit):")? {
                if let Some(item) = choice
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| index.checked_sub(1))
                    .and_then(|index| items.get(index))
                {
                    let article = request(
                        cli,
                        json!({"command":"docs.show","id":s(item,"id"),"stack":s(item,"stack"),"version":s(item,"version")}),
                    )?;
                    read_article(&article, false)?;
                    break;
                }
                eprintln!("Enter a number from 1 to {}.", items.len());
            }
        } else if io::stdout().is_terminal() {
            println!("Read a result with devline docs show '<result-id>', or rerun with --pick.");
        }
    }
    Ok(if items.is_empty() { 4 } else { 0 })
}
fn run(cli: &Cli) -> Result<i32> {
    let (action, data) = match &cli.command {
        None => {
            Cli::command().print_help()?;
            println!();
            return Ok(0);
        }
        Some(Commands::Completions { shell }) => {
            print_completions(*shell)?;
            return Ok(0);
        }
        Some(Commands::Library) => ("catalog", request(cli, json!({"command":"catalog"}))?),
        Some(Commands::Runtimes {
            command: None | Some(Runtimes::List),
        }) => (
            "runtime-list",
            request(cli, json!({"command":"runtime.list"}))?,
        ),
        Some(Commands::Updates { command }) => match command {
            None | Some(Updates::Status) => (
                "updates",
                request(cli, json!({"command":"updates.status"}))?,
            ),
            Some(Updates::Check) => {
                if cli.offline {
                    bail!("Checking for updates requires a connection. Run devline updates to read saved status offline.");
                }
                progress(cli, format_args!("Checking upstream update sources…"));
                ("updates", request(cli, json!({"command":"updates.check"}))?)
            }
            Some(Updates::Configure { url, clear }) => {
                let url = if *clear {
                    ""
                } else {
                    url.as_deref().unwrap_or_default().trim()
                };
                if !*clear && url.is_empty() {
                    bail!("Enter a public HTTPS manifest URL, or use devline updates configure --clear.");
                }
                (
                    if *clear {
                        "updates-cleared"
                    } else {
                        "updates-configured"
                    },
                    request(cli, json!({"command":"updates.configure","url":url}))?,
                )
            }
        },
        Some(Commands::Docs { command }) => match command {
            None => {
                let query = ask("Search installed docs:")?;
                return search_docs(cli, &query, None, None, 20, !cli.json);
            }
            Some(Docs::Search {
                query,
                stack,
                version,
                limit,
                pick,
            }) => {
                return search_docs(
                    cli,
                    query,
                    stack.as_deref(),
                    version.as_deref(),
                    *limit,
                    *pick,
                )
            }
            Some(Docs::Show {
                id,
                stack,
                version,
                no_pager,
                open,
            }) => {
                let result = request(
                    cli,
                    json!({"command":"docs.show","id":id,"stack":stack,"version":version}),
                )?;
                if cli.json {
                    println!("{}", json!({"schema_version":1,"data":result}));
                } else if *open {
                    let encoded = s(&result, "id")
                        .bytes()
                        .map(|b| format!("%{b:02X}"))
                        .collect::<String>();
                    let status = Command::new("/usr/bin/open")
                        .arg(format!("devkit://docs?id={encoded}"))
                        .status()
                        .context("Could not open Devline")?;
                    if !status.success() {
                        bail!("Open Devline.app once to register documentation links");
                    }
                    println!("{}", s(&result, "title"));
                } else {
                    read_article(&result, *no_pager)?;
                }
                return Ok(0);
            }
            Some(Docs::Install { id, version }) => {
                progress(cli, format_args!("Installing documentation for {id}…"));
                (
                    "install",
                    request(
                        cli,
                        json!({"command":"docs.install","id":id,"version":version}),
                    )?,
                )
            }
            Some(Docs::List { installed }) => {
                let mut list = request(cli, json!({"command":"catalog"}))?["docs"].clone();
                if *installed {
                    list.as_array_mut()
                        .unwrap()
                        .retain(|v| v["installed"] == true);
                }
                ("docs-list", list)
            }
            Some(Docs::Remove { id, version }) => (
                "remove",
                request(
                    cli,
                    json!({"command":"docs.remove","id":id,"version":version}),
                )?,
            ),
        },
        Some(Commands::Templates { command }) => match command {
            Templates::List {
                installed,
                app_type,
                stack,
            } => {
                let mut list = request(cli, json!({"command":"templates.list"}))?;
                list.as_array_mut()
                    .context("Unexpected template list")?
                    .retain(|v| {
                        (!installed || v["installed"] == true)
                            && app_type.as_ref().is_none_or(|a| v["app_type"] == *a)
                            && stack.as_ref().is_none_or(|a| v["stack"] == *a)
                    });
                ("template-list", list)
            }
            Templates::Install { id } => {
                progress(cli, format_args!("Installing starter {id}…"));
                (
                    "install",
                    request(cli, json!({"command":"templates.install","id":id}))?,
                )
            }
            Templates::Build { id, .. } => {
                if cli.offline {
                    bail!("Building with latest upstream dependencies requires a connection. Run this command while connected.");
                }
                progress(cli, format_args!("Refreshing dependencies for {id}, then testing with network access blocked…"));
                (
                    "build",
                    request(cli, json!({"command":"templates.build","id":id}))?,
                )
            }
        },
        Some(Commands::Prepare { id }) => {
            progress(
                cli,
                format_args!("Installing starter and matching documentation for {id}…"),
            );
            (
                "prepare",
                request(cli, json!({"command":"prepare","id":id}))?,
            )
        }
        Some(Commands::Doctor { id }) => {
            ("doctor", request(cli, json!({"command":"doctor","id":id}))?)
        }
        Some(Commands::New {
            id,
            destination,
            dry_run,
        }) => {
            let selected = match id {
                Some(id) => id.clone(),
                None => {
                    let list = request(cli, json!({"command":"templates.list"}))?;
                    let items: Vec<_> = list
                        .as_array()
                        .context("Unexpected list")?
                        .iter()
                        .filter(|v| v["installed"] == true)
                        .collect();
                    if items.is_empty() {
                        bail!("No starters are installed. Browse with devline templates list, then install with devline prepare <id>.");
                    }
                    for (i, v) in items.iter().enumerate() {
                        eprintln!("{}. {}@{}", i + 1, s(v, "id"), s(v, "version"));
                    }
                    let number: usize = ask("Choose a starter number:")?
                        .parse()
                        .context("Choose a listed number")?;
                    let v = items
                        .get(number.checked_sub(1).context("Choose a listed number")?)
                        .context("Choose a listed number")?;
                    format!("{}@{}", s(v, "id"), s(v, "version"))
                }
            };
            let path = match destination {
                Some(p) => p.clone(),
                None => PathBuf::from(ask("Project directory (for example ./status-api):")?),
            };
            (
                "new",
                request(
                    cli,
                    json!({"command":"new","id":selected,"destination":path,"dry_run":dry_run}),
                )?,
            )
        }
    };
    let ready = match action {
        "doctor" => data["ready"].as_bool(),
        "prepare" => data["doctor"]["ready"].as_bool(),
        _ => None,
    };
    if cli.json {
        println!("{}", json!({"schema_version":1,"data":data}));
    } else {
        match action {
            "runtime-list" => render_runtimes(&data)?,
            "updates" => render_updates(cli, &data)?,
            "updates-configured" => println!(
                "App update channel saved.\nRun {} updates check when you’re ready to connect.",
                update_command_prefix(cli)
            ),
            "updates-cleared" => {
                println!("App update channel cleared. Devline will show Local build.")
            }
            "docs-list" => render_packages(
                &data,
                "No matching docs packs. Run devline docs list to see the full catalog.",
            ),
            "template-list" => render_packages(
                &data,
                "No matching starter releases. Run devline templates list to browse all releases.",
            ),
            "catalog" => {
                println!("Documentation\n");
                render_packages(&data["docs"], "No docs packs available.");
                println!("\nTemplates\n");
                render_packages(&data["templates"], "No starter releases available.");
                println!("\nLibrary  {}", s(&data, "home"));
            }
            "doctor" => readiness(&data),
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
                let release = format!("{}@{}", s(&data, "template"), s(&data, "version"));
                let path = s(&data, "path");
                if dry_run {
                    println!("Ready to create {path}\nTemplate  {release}\n\nCreate the project\n  devline new {release} {}", shell_quote(path));
                } else {
                    println!(
                        "Created {path}\nTemplate  {release}\n\nTest offline\n  cd {}\n  {}",
                        shell_quote(path),
                        s(&data, "test_command")
                    );
                }
            }
            "install" => println!(
                "Installed {}@{}{}",
                s(&data, "id"),
                s(&data, "version"),
                data.get("count")
                    .map(|c| format!(" · {c} documents"))
                    .unwrap_or_default()
            ),
            "remove" => println!("Documentation removed from the local library."),
            "build" => println!(
                "Built and installed {}@{}\nPassed network-denied verification.\n{}",
                s(&data, "id"),
                s(&data, "version"),
                s(&data, "path")
            ),
            _ => println!("{}", serde_json::to_string_pretty(&data)?),
        }
    }
    if ready == Some(false) {
        return Ok(3);
    }
    Ok(0)
}
fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}
fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let cli = Cli::parse();
    if cli.command.is_none() {
        if let Ok(value) = std::env::var("_DEVKIT_ZSH_COMPLETION") {
            if let Ok(kind) = CompletionKind::from_str(&value, false) {
                std::process::exit(if completion_values(&cli, kind).is_ok() {
                    0
                } else {
                    1
                });
            }
        }
    }
    match run(&cli) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            if cli.json {
                eprintln!(
                    "{}",
                    json!({"schema_version":1,"error":{"code":"operation_failed","message":format!("{e:#}")}})
                );
            } else {
                eprintln!("devline: {e:#}");
            }
            std::process::exit(1);
        }
    }
}
