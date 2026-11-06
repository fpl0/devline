use anyhow::{bail, Context, Result};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

mod terminal;

#[derive(Parser)]
#[command(
    name = "devline",
    version,
    about = "Your offline developer library",
    long_about = "Search official docs and create projects from installed starters. The CLI shares its local library with Devline.app and works with the app closed.",
    after_long_help = "Examples:\n  devline library                               Browse bundled docs and starters\n  devline runtimes                              Check toolchains on this Mac\n  devline docs install go                       Add the Go reference to your library\n  devline docs search 'go:http.Handler' --pick  Search and read in the terminal\n  devline docs save 'go:http.Handler'           Save a reference across app and CLI\n  devline prepare go/chi-api                     Install the starter and matching docs\n  devline new go/chi-api ./status-api            Create a project offline\n  devline updates                               Read saved update status\n  devline updates check                         Check upstream sources online\n\nRun 'devline completions --help' for shell setup. The previous 'devkit' command remains a compatibility alias."
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
        after_long_help = "Run 'devline docs' for interactive search. Prefix a query with a docset, as in 'go:http.Handler', or use --docset. With a single detected stack in the current project, search defaults to that stack. Use --stack all to search the full installed library."
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
#[derive(Args, Default)]
struct Scope {
    #[arg(long)]
    /// Search a stack such as go or dotnet, or use all.
    stack: Option<String>,
    #[arg(long)]
    /// Match an exact documentation release.
    version: Option<String>,
    #[arg(long, value_name = "PACK[@VERSION]")]
    /// Search a docset such as fastapi, optionally pinned to a release.
    docset: Option<String>,
}

#[derive(Subcommand)]
enum Docs {
    #[command(about = "Find symbols and articles in installed packs")]
    Search {
        query: String,
        #[command(flatten)]
        scope: Scope,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        /// Choose a result and read it in the terminal.
        pick: bool,
        #[arg(long, conflicts_with = "pick")]
        /// Print one exact article ID per line, suitable for fzf.
        ids: bool,
    },
    #[command(
        about = "Read an article by symbol or exact result ID",
        after_long_help = "Examples:\n  devline docs show 'go:http.Handler'\n  devline docs show 'go:http.Handler' --open\n  devline docs show 'go:http.Handler' --link\n\nRedirect output or use --no-pager for the raw article. Set DEVLINE_PAGER or PAGER to choose an interactive pager."
    )]
    Show {
        id: String,
        #[command(flatten)]
        scope: Scope,
        #[arg(long)]
        /// Print the raw article directly to stdout.
        no_pager: bool,
        #[arg(long)]
        /// Open the article in Devline.app.
        open: bool,
        #[arg(long, conflicts_with_all = ["open", "no_pager"])]
        /// Print the local Devline link without opening the app.
        link: bool,
    },
    #[command(about = "List saved references shared with Devline.app")]
    Saved {
        #[arg(long)]
        /// Choose and reopen a saved reference in the terminal.
        pick: bool,
    },
    #[command(about = "List recently opened references")]
    Recent {
        #[arg(long)]
        /// Choose and reopen a recent reference in the terminal.
        pick: bool,
    },
    #[command(about = "Save an article or symbol for offline reading")]
    Save {
        id: String,
        #[command(flatten)]
        scope: Scope,
    },
    #[command(about = "Remove a saved reference without removing its documentation")]
    Unsave {
        id: String,
        #[command(flatten)]
        scope: Scope,
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
    InstalledDocsets,
    Stacks,
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
    devkit_core::execute(&request)
}

fn exact_id(id: &str) -> bool {
    id.split_once('/')
        .is_some_and(|(release, _)| release.contains('@'))
}

impl Scope {
    fn apply(&self, request: &mut Value) -> Result<()> {
        let mut version = self.version.as_deref();
        if let Some(docset) = &self.docset {
            let (pack, pin) = docset
                .split_once('@')
                .map_or((docset.as_str(), None), |(pack, pin)| (pack, Some(pin)));
            if pack.is_empty() || pin.is_some_and(str::is_empty) {
                bail!("Use --docset PACK or --docset PACK@VERSION.");
            }
            if pin.is_some() && version.is_some() && pin != version {
                bail!("--docset and --version specify different releases.");
            }
            version = pin.or(version);
            request["pack_id"] = json!(pack);
        }
        request["version"] = json!(version);
        if let Some(stack) = &self.stack {
            if stack != "all" {
                request["stack"] = json!(stack);
            }
        } else if self.docset.is_none()
            && version.is_none()
            && !request["id"].as_str().is_some_and(exact_id)
        {
            if let Ok(cwd) = std::env::current_dir() {
                request["project_stack"] = devkit_core::project_context(&cwd)["stack"].clone();
            }
        }
        Ok(())
    }
}

fn completion_values(cli: &Cli, kind: CompletionKind) -> Result<()> {
    let (docs, installed) = match kind {
        CompletionKind::AvailableDocs | CompletionKind::Stacks => (true, false),
        CompletionKind::InstalledDocs | CompletionKind::InstalledDocsets => (true, true),
        CompletionKind::AvailableStarters => (false, false),
        CompletionKind::InstalledStarters => (false, true),
    };
    let data = if docs {
        request(cli, json!({"command":"catalog"}))?["docs"].clone()
    } else {
        request(cli, json!({"command":"templates.list"}))?
    };
    let mut choices = BTreeSet::new();
    let stacks = matches!(kind, CompletionKind::Stacks);
    if stacks {
        choices.insert("all".to_owned());
    }
    for item in data
        .as_array()
        .context("Unexpected completion candidates")?
    {
        if installed && item["installed"] != true {
            continue;
        }
        let id = s(item, if stacks { "stack" } else { "id" });
        if id.is_empty()
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_/".contains(&b))
        {
            continue;
        }
        choices.insert(id.to_owned());
        let version = s(item, "version");
        if (!docs || matches!(kind, CompletionKind::InstalledDocsets))
            && !version.is_empty()
            && version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
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

fn completion_scope(command: clap::Command) -> clap::Command {
    command
        .mut_arg("docset", |argument| {
            argument.value_parser(["__installed_docsets"])
        })
        .mut_arg("stack", |argument| argument.value_parser(["__stacks"]))
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
                .mut_subcommand("search", completion_scope)
                .mut_subcommand("show", completion_scope)
                .mut_subcommand("save", completion_scope)
                .mut_subcommand("unsave", completion_scope)
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
            "(__installed_docsets)",
            "_devline_local_ids installed-docsets",
        )
        .replace("(__stacks)", "_devline_local_ids stacks")
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
fn resolve_article(cli: &Cli, input: &str, scope: &Scope, terminal_format: bool) -> Result<Value> {
    let mut query = json!({"command":"docs.show","id":input});
    scope.apply(&mut query)?;
    if terminal_format {
        query["terminal_format"] = json!(true);
    }
    request(cli, query)
}

fn reading_action(cli: &Cli, command: &str, article: &Value) -> Result<Value> {
    request(
        cli,
        json!({
            "command":command,
            "id":article["id"],
            "anchor":article["anchor"],
            "focus_text":article["focus_text"],
        }),
    )
}

fn read_article(cli: &Cli, article: &Value, no_pager: bool) -> Result<()> {
    if io::stdout().is_terminal() {
        if let Err(error) = reading_action(cli, "reading.visit", article) {
            progress(cli, format_args!("Could not record this visit: {error}"));
        }
    }
    terminal::read(article, no_pager)
}

fn reference_link(article: &Value) -> String {
    use std::fmt::Write as _;
    let mut url = String::from("devline://docs?");
    for (name, key) in [("id", "id"), ("anchor", "anchor"), ("focus", "focus_text")] {
        let Some(value) = article[key].as_str().filter(|value| !value.is_empty()) else {
            continue;
        };
        if !url.ends_with('?') {
            url.push('&');
        }
        url.push_str(name);
        url.push('=');
        for byte in value.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                url.push(byte as char);
            } else {
                let _ = write!(url, "%{byte:02X}");
            }
        }
    }
    url
}

fn render_reading_list(data: &Value, saved: bool, pick: bool) -> Result<()> {
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

fn require_interactive_picker(cli: &Cli) -> Result<()> {
    if cli.json {
        bail!("Choose --pick for interactive reading or --json for structured results.");
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() || !io::stderr().is_terminal() {
        bail!("--pick needs an interactive terminal.");
    }
    Ok(())
}

fn pick_article(cli: &Cli, items: &[Value], query: Option<&str>) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    while let Some(choice) = answer("Open result number (Return to exit):")? {
        let item = choice
            .parse::<usize>()
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| items.get(index));
        let Some(item) = item else {
            eprintln!("Enter a number from 1 to {}.", items.len());
            continue;
        };
        if item["installed"] == false {
            eprintln!(
                "Install {}@{} to reopen this reference.",
                s(item, "pack_id"),
                s(item, "version")
            );
            continue;
        }
        let focus = item["focus_text"].as_str().or(item["match_text"].as_str());
        let mut article = request(
            cli,
            json!({
                "command":"docs.show", "id":s(item,"id"), "query":query, "focus_text":focus,
                "terminal_format":terminal::interactive(false)
            }),
        )?;
        if article["focus_text"].is_null() {
            article["focus_text"] = json!(focus);
        }
        if article["anchor"].is_null() {
            article["anchor"] = item["anchor"].clone();
        }
        return read_article(cli, &article, false);
    }
    Ok(())
}

fn search_docs(
    cli: &Cli,
    query: &str,
    scope: &Scope,
    limit: u32,
    pick: bool,
    ids: bool,
) -> Result<i32> {
    if pick {
        require_interactive_picker(cli)?;
    }
    if ids && cli.json {
        bail!("Choose --ids for exact article IDs or --json for structured results.");
    }
    let mut search = json!({"command":"docs.search","query":query,"limit":limit});
    scope.apply(&mut search)?;
    let inferred = search["project_stack"]
        .as_str()
        .filter(|_| {
            query
                .split_once(':')
                .is_none_or(|(_, suffix)| suffix.starts_with(':'))
        })
        .map(str::to_owned);
    let data = request(cli, search)?;
    let items = data.as_array().context("Unexpected search results")?;
    if cli.json {
        println!("{}", json!({"schema_version":1,"data":data}));
    } else if ids {
        for item in items {
            println!("{}", s(item, "id"));
        }
    } else if items.is_empty() {
        if let Some(stack) = &inferred {
            println!(
                "No installed documentation matched {query:?} in the detected {stack} project."
            );
            println!("Add --stack all to search every installed docset.");
        } else {
            println!("No installed documentation matched {query:?}.");
            println!("Try a broader term, or remove the docset and version filters.");
        }
        println!(
            "Run devline docs list to see available packs and devline docs install <id> to add one."
        );
    } else {
        if let Some(stack) = &inferred {
            progress(
                cli,
                format_args!(
                    "Project scope is {stack}. Use --stack all to search all installed docsets."
                ),
            );
        }
        for (index, item) in items.iter().enumerate() {
            if pick {
                print!("{:>2}. ", index + 1);
            }
            let title = terminal::safe_text(s(item, "title"));
            if let Some(symbol) = item["symbol"].as_str().filter(|symbol| *symbol != title) {
                print!("{} · ", terminal::safe_text(symbol));
            }
            println!(
                "{title}  [{}@{}]\n  {}\n  {}\n",
                s(item, "pack_id"),
                s(item, "version"),
                s(item, "id"),
                terminal::safe_text(s(item, "snippet")).replace('\n', " ")
            );
        }
        if pick {
            pick_article(cli, items, Some(query))?;
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
                return search_docs(cli, &query, &Scope::default(), 20, !cli.json, false);
            }
            Some(Docs::Search {
                query,
                scope,
                limit,
                pick,
                ids,
            }) => {
                return search_docs(cli, query, scope, *limit, *pick, *ids);
            }
            Some(Docs::Show {
                id,
                scope,
                no_pager,
                open,
                link,
            }) => {
                if cli.json && *link {
                    bail!("Choose --link for the local URL or --json for the article.");
                }
                let terminal_format =
                    !cli.json && !open && !link && terminal::interactive(*no_pager);
                let result = resolve_article(cli, id, scope, terminal_format)?;
                if cli.json {
                    println!("{}", json!({"schema_version":1,"data":result}));
                } else if *open {
                    let status = Command::new("/usr/bin/open")
                        .arg(reference_link(&result))
                        .status()
                        .context("Could not open Devline")?;
                    if !status.success() {
                        bail!("Open Devline.app once to register documentation links");
                    }
                    println!("{}", terminal::safe_text(s(&result, "title")));
                } else if *link {
                    println!("{}", reference_link(&result));
                } else {
                    read_article(cli, &result, *no_pager)?;
                }
                return Ok(0);
            }
            Some(Docs::Saved { pick } | Docs::Recent { pick }) => {
                if *pick {
                    require_interactive_picker(cli)?;
                }
                let saved = matches!(command, Some(Docs::Saved { .. }));
                let key = if saved { "saved" } else { "recent" };
                let mut result = request(cli, json!({"command":"reading.list"}))?;
                if *pick {
                    render_reading_list(&result[key], saved, true)?;
                    pick_article(
                        cli,
                        result[key].as_array().context("Unexpected reading list")?,
                        None,
                    )?;
                    return Ok(0);
                }
                (key, result[key].take())
            }
            Some(Docs::Save { id, scope }) => {
                let article = resolve_article(cli, id, scope, false)?;
                (
                    "saved-article",
                    reading_action(cli, "reading.save", &article)?,
                )
            }
            Some(Docs::Unsave { id, scope }) => {
                let id = if exact_id(id) {
                    id.clone()
                } else {
                    s(&resolve_article(cli, id, scope, false)?, "id").to_owned()
                };
                let result = request(cli, json!({"command":"reading.remove","id":id}))?;
                ("unsaved-article", result)
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
            "saved" => render_reading_list(&data, true, false)?,
            "recent" => render_reading_list(&data, false, false)?,
            "saved-article" => {
                println!("Saved reference. Find it in Devline or with devline docs saved.")
            }
            "unsaved-article" => println!("Saved reference removed."),
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
