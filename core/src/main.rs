use anyhow::{bail, Context, Result};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod cli_infra;
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
        help = "Require local resources and skip network preparation"
    )]
    offline: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    #[command(
        about = "Search and read installed documentation",
        after_long_help = "Run 'devline docs' for interactive search. Unscoped search selects installed releases compatible with the current project. Run 'devline project' to inspect that selection. Prefix a query with a docset, use --docset, or use --stack all to override project scope."
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
    #[command(
        about = "Inspect project dependencies and pin documentation",
        after_long_help = "Examples:\n  devline project\n  devline project --path ./services/api\n  devline project pin fastapi@0.162.0+devline.1\n  devline project unpin fastapi"
    )]
    Project {
        #[arg(long, global = true, value_hint = clap::ValueHint::AnyPath)]
        /// Project folder or file. Defaults to the current directory.
        path: Option<PathBuf>,
        #[command(subcommand)]
        command: Option<Project>,
    },
    #[command(about = "Inspect offline readiness and verify installed starters")]
    Offline {
        #[command(subcommand)]
        command: Option<Offline>,
    },
    #[command(about = "Check a project's docs, starter, toolchains and image vault together")]
    Toolbox(cli_infra::Toolbox),
    #[command(
        about = "Archive container images and run project services offline",
        after_long_help = "Start with 'devline infra catalog' to find an exact release. Prepare it while connected, then configure a project and use 'devline infra run'. Run never pulls images. Use 'devline infra export' and 'devline infra import' to transfer archives."
    )]
    Infra {
        #[command(subcommand)]
        command: Option<cli_infra::Infra>,
    },
    #[command(about = "Export, inspect, and import an offline library bundle")]
    Bundle {
        #[command(subcommand)]
        command: Bundle,
    },
    #[command(about = "Inspect and cancel background operations")]
    Jobs {
        #[command(subcommand)]
        command: Option<Jobs>,
    },
    #[command(
        about = "Inspect local runtimes and toolchains",
        long_about = "Report the executables and versions discovered on this Mac, from PATH and known locations such as Homebrew, mise and rustup. Starter minimums are listed separately. Use the runtime's own tools to install or upgrade it."
    )]
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
        #[arg(long)]
        /// Start a cancellable job and print its ID.
        background: bool,
    },
    #[command(about = "Check a starter and its toolchain on this Mac")]
    Doctor { id: String },
    #[command(
        about = "Create a project from an installed starter",
        after_long_help = "Examples:\n  devline new go/chi-api ./status-api\n  devline new go/chi-api@1.0.0 ./status-api --dry-run\n  devline --offline new go/chi-taskboard ./board\n\nDevline installs the selected starter, tests the generated project with external networking denied, and configures requested services. Missing service images are prepared while connected. Use --offline to require their existing archives. Without arguments, Devline prompts for a starter and destination."
    )]
    New {
        /// Starter ID or exact release.
        id: Option<String>,
        /// Destination directory for the generated project.
        destination: Option<PathBuf>,
        #[arg(long)]
        /// Check the selected release and destination before creating files.
        dry_run: bool,
        /// Add an exact local service release. Repeat for more services.
        #[arg(long = "service", value_name = "SERVICE@VERSION", value_parser = cli_infra::release)]
        services: Vec<String>,
        /// Override a service's published localhost port.
        #[arg(long = "port", value_name = "SERVICE:PORT=HOST_PORT", value_parser = cli_infra::port)]
        ports: Vec<cli_infra::Port>,
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
    #[arg(long, value_hint = clap::ValueHint::AnyPath)]
    /// Resolve project versions from this folder or file.
    project: Option<PathBuf>,
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
        #[arg(long)]
        /// Install from the bundled catalog as a background job.
        background: bool,
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
    #[command(about = "View or check the independently published documentation channel")]
    Updates {
        #[command(subcommand)]
        command: Option<DocsUpdates>,
    },
    #[command(about = "Download and activate an exact documentation release")]
    Fetch {
        /// Pack ID or PACK@VERSION. Omit the version for the latest checked release.
        id: String,
        #[arg(long)]
        /// Download as a cancellable background job.
        background: bool,
    },
    #[command(about = "Activate an earlier installed documentation release")]
    Rollback {
        /// Pack ID or PACK@VERSION. Omit the version to use the previous release.
        id: String,
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
        #[arg(long)]
        /// Start a cancellable job and print its ID.
        background: bool,
    },
    #[command(about = "Inspect a starter's files, dependencies, and verification")]
    Inspect { id: String },
    #[command(about = "Test an installed starter with external networking denied")]
    Verify {
        id: String,
        #[arg(long)]
        /// Start a cancellable job and print its ID.
        background: bool,
    },
    #[command(about = "Remove one exact installed starter release")]
    Remove { id: String },
}

#[derive(Subcommand)]
enum Project {
    #[command(about = "Show detected dependencies and selected documentation")]
    Status,
    #[command(about = "Check the current files and toolchain against an offline test receipt")]
    Readiness,
    #[command(about = "Prepare locked dependencies and verify the existing project")]
    Prepare {
        /// Start a cancellable job and print its ID.
        #[arg(long)]
        background: bool,
    },
    #[command(about = "Run a command using this project's prepared offline dependencies")]
    Run {
        #[arg(required = true, last = true, value_name = "COMMAND")]
        command: Vec<std::ffi::OsString>,
    },
    #[command(about = "Pin an exact documentation release for this project")]
    Pin {
        #[arg(value_name = "PACK@VERSION")]
        id: String,
    },
    #[command(about = "Restore automatic version selection for a project docset")]
    Unpin { id: String },
}

#[derive(Subcommand)]
enum DocsUpdates {
    #[command(about = "Show the cached documentation channel state")]
    Status,
    #[command(about = "Fetch the documentation release manifest")]
    Check,
    #[command(about = "Set or clear the independent documentation channel")]
    Configure {
        #[arg(required_unless_present = "clear", conflicts_with = "clear", value_hint = clap::ValueHint::Url)]
        url: Option<String>,
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Args)]
struct BundleSelection {
    #[arg(long = "doc", value_name = "PACK@VERSION")]
    /// Select an exact documentation release. Repeat to select more.
    docs: Vec<String>,
    #[arg(long = "starter", value_name = "STARTER@VERSION")]
    /// Select an exact starter release. Repeat to select more.
    starters: Vec<String>,
}

#[derive(Subcommand)]
enum Offline {
    #[command(about = "Report installed documentation and starter readiness")]
    Status {
        #[arg(long, value_hint = clap::ValueHint::AnyPath)]
        path: Option<PathBuf>,
        #[command(flatten)]
        selection: BundleSelection,
    },
    #[command(about = "Run the network-denied smoke test for an installed starter")]
    Check {
        id: String,
        #[arg(long)]
        background: bool,
    },
}

#[derive(Subcommand)]
enum Bundle {
    #[command(about = "Export selected pinned releases to a portable bundle")]
    Export {
        #[arg(value_hint = clap::ValueHint::FilePath)]
        path: PathBuf,
        #[command(flatten)]
        selection: BundleSelection,
        #[arg(long)]
        background: bool,
    },
    #[command(about = "Validate a bundle and show what it contains")]
    Inspect {
        #[arg(value_hint = clap::ValueHint::FilePath)]
        path: PathBuf,
    },
    #[command(about = "Install the verified contents of a local bundle")]
    Import {
        #[arg(value_hint = clap::ValueHint::FilePath)]
        path: PathBuf,
        #[arg(long)]
        /// Trust and install executable starter files from this bundle.
        trust_starters: bool,
        #[arg(long)]
        background: bool,
    },
}

#[derive(Subcommand)]
enum Jobs {
    #[command(about = "List background jobs")]
    List,
    #[command(about = "Read a job status and log")]
    Status {
        id: String,
        #[arg(long, default_value_t = 0)]
        /// Read the log from this byte offset.
        offset: u64,
        #[arg(long)]
        /// Stream log updates until the job stops.
        follow: bool,
    },
    #[command(about = "Request cancellation of a background job")]
    Cancel { id: String },
}
#[derive(Subcommand)]
enum Runtimes {
    #[command(
        about = "Report detected tools, versions and starter minimums",
        long_about = "Probe the local toolchain executables and compare their versions to the selected stack's starter minimums. Devline looks on PATH and in known macOS tool locations, including Homebrew, mise and rustup. Runtime installation and upgrades stay with the runtime's own tool or installer."
    )]
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
    ExactDocs,
    Stacks,
    AvailableStarters,
    InstalledStarters,
    ExactStarters,
    InfraImages,
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
            request["stack"] = json!(stack);
        } else if self.docset.is_none()
            && version.is_none()
            && !request["id"].as_str().is_some_and(exact_id)
        {
            if let Some(path) = self
                .project
                .clone()
                .or_else(|| std::env::current_dir().ok())
            {
                request["project_path"] = json!(path);
            }
        }
        Ok(())
    }
}

fn completion_values(cli: &Cli, kind: CompletionKind) -> Result<()> {
    let (docs, installed) = match kind {
        CompletionKind::AvailableDocs | CompletionKind::Stacks => (true, false),
        CompletionKind::InstalledDocs
        | CompletionKind::InstalledDocsets
        | CompletionKind::ExactDocs => (true, true),
        CompletionKind::AvailableStarters => (false, false),
        CompletionKind::InstalledStarters | CompletionKind::ExactStarters => (false, true),
        CompletionKind::InfraImages => return cli_infra::complete_images(cli),
    };
    let data = if docs {
        request(cli, json!({"command":"catalog"}))?["docs"].clone()
    } else {
        request(cli, json!({"command":"templates.list"}))?
    };
    let mut choices = BTreeSet::new();
    let stacks = matches!(kind, CompletionKind::Stacks);
    let exact = matches!(
        kind,
        CompletionKind::ExactDocs | CompletionKind::ExactStarters
    );
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
        if !exact {
            choices.insert(id.to_owned());
        }
        let version = s(item, "version");
        if (!docs
            || matches!(
                kind,
                CompletionKind::InstalledDocsets | CompletionKind::ExactDocs
            ))
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
                .mut_subcommand("rollback", |cmd| {
                    completion_argument(cmd, "__installed_docsets")
                })
        })
        .mut_subcommand("templates", |command| {
            command
                .mut_subcommand("install", |cmd| {
                    completion_argument(cmd, "__available_starters")
                })
                .mut_subcommand("build", |cmd| {
                    completion_argument(cmd, "__available_starters")
                })
                .mut_subcommand("inspect", |cmd| {
                    completion_argument(cmd, "__available_starters")
                })
                .mut_subcommand("verify", |cmd| {
                    completion_argument(cmd, "__installed_starters")
                })
                .mut_subcommand("remove", |cmd| completion_argument(cmd, "__exact_starters"))
        })
        .mut_subcommand("project", |cmd| {
            cmd.mut_subcommand("pin", |cmd| completion_argument(cmd, "__exact_docs"))
                .mut_subcommand("unpin", |cmd| completion_argument(cmd, "__installed_docs"))
        })
        .mut_subcommand("offline", |cmd| {
            cmd.mut_subcommand("check", |cmd| {
                completion_argument(cmd, "__installed_starters")
            })
        })
        .mut_subcommand("bundle", |cmd| {
            cmd.mut_subcommand("export", |cmd| {
                cmd.mut_arg("docs", |arg| arg.value_parser(["__exact_docs"]))
                    .mut_arg("starters", |arg| arg.value_parser(["__exact_starters"]))
            })
        })
        .mut_subcommand("infra", |command| {
            command
                .mut_subcommand("prepare", |cmd| completion_argument(cmd, "__infra_images"))
                .mut_subcommand("restore", |cmd| completion_argument(cmd, "__infra_images"))
                .mut_subcommand("verify", |cmd| completion_argument(cmd, "__infra_images"))
                .mut_subcommand("remove", |cmd| completion_argument(cmd, "__infra_images"))
                .mut_subcommand("status", |cmd| completion_argument(cmd, "__infra_images"))
                .mut_subcommand("export", |cmd| {
                    cmd.mut_arg("images", |arg| arg.value_parser(["__infra_images"]))
                })
                .mut_subcommand("configure", |cmd| {
                    cmd.mut_arg("services", |arg| arg.value_parser(["__infra_images"]))
                })
        })
        .mut_subcommand("doctor", |cmd| {
            completion_argument(cmd, "__available_starters")
        })
        .mut_subcommand("prepare", |cmd| {
            completion_argument(cmd, "__available_starters")
        })
        .mut_subcommand("new", |cmd| {
            completion_argument(cmd, "__available_starters")
                .mut_arg("services", |arg| arg.value_parser(["__infra_images"]))
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
        .replace("(__exact_docs)", "_devline_local_ids exact-docs")
        .replace("(__exact_starters)", "_devline_local_ids exact-starters")
        .replace("(__infra_images)", "_devline_local_ids infra-images")
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
    let project = query["project_path"]
        .as_str()
        .filter(|_| may_use_project(input))
        .map(str::to_owned);
    request(cli, query).map_err(|error| {
        let Some(path) = project else {
            return error;
        };
        let Ok(context) = request(cli, json!({"command":"project.context","path":path})) else {
            return error;
        };
        let details: Vec<_> = project_warnings(&context).collect();
        if details.is_empty() {
            error
        } else {
            error.context(format!("Project documentation: {}", details.join(" ")))
        }
    })
}

fn may_use_project(query: &str) -> bool {
    query
        .split_once(':')
        .is_none_or(|(_, suffix)| suffix.starts_with([':', '/']))
}

fn project_warnings(context: &Value) -> impl Iterator<Item = &str> {
    context["docsets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|pack| {
            matches!(pack["state"].as_str(), Some("unmatched" | "uninstalled"))
                || (pack["state"] == "pinned"
                    && (pack["compatible"] == false
                        || !pack["installed_versions"]
                            .as_array()
                            .is_some_and(|versions| versions.contains(&pack["version"]))))
        })
        .filter_map(|pack| pack["detail"].as_str())
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

fn pack_reference(input: &str, required: bool) -> Result<(&str, Option<&str>)> {
    let (id, version) = input
        .split_once('@')
        .map_or((input, None), |(id, version)| (id, Some(version)));
    if id.is_empty()
        || version.is_some_and(|version| version.is_empty() || version.contains('@'))
        || (required && version.is_none())
    {
        bail!("Use an exact release such as fastapi@0.162.0+devline.1.");
    }
    Ok((id, version))
}

fn run_operation(
    cli: &Cli,
    action: &'static str,
    kind: &str,
    background: bool,
    mut payload: Value,
) -> Result<(&'static str, Value)> {
    if background {
        payload["command"] = json!("jobs.start");
        payload["kind"] = json!(kind);
        return Ok(("job-started", request(cli, payload)?));
    }
    Ok((action, request(cli, payload)?))
}

fn render_job(job: &Value) {
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

fn watch_job(cli: &Cli, id: &str, mut offset: u64) -> Result<i32> {
    loop {
        let job = request(
            cli,
            json!({"command":"jobs.status","job_id":id,"offset":offset}),
        )?;
        if !cli.json {
            eprint!("{}", terminal::safe_text(s(&job, "log")));
        }
        let next = job["next_offset"].as_u64().unwrap_or(offset);
        let terminal = !matches!(job["state"].as_str(), Some("queued" | "running"));
        if terminal && next == offset {
            if cli.json {
                println!("{}", json!({"schema_version":1,"data":job}));
            } else {
                render_job(&job);
            }
            return Ok(match job["state"].as_str() {
                Some("succeeded") => 0,
                Some("cancelled") => 130,
                _ => 1,
            });
        }
        if next == offset {
            std::thread::sleep(Duration::from_millis(250));
        }
        offset = next;
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

fn render_offline(value: &Value) {
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
    let project = search["project_path"]
        .as_str()
        .filter(|_| may_use_project(query))
        .map(str::to_owned);
    let data = request(cli, search)?;
    let items = data.as_array().context("Unexpected search results")?;
    let inferred = project
        .filter(|_| !cli.json && !ids && (items.is_empty() || io::stderr().is_terminal()))
        .and_then(|path| request(cli, json!({"command":"project.context","path":path})).ok())
        .filter(|context| {
            context["docsets"]
                .as_array()
                .is_some_and(|packs| !packs.is_empty())
        });
    if cli.json {
        println!("{}", json!({"schema_version":1,"data":data}));
    } else if ids {
        for item in items {
            println!("{}", s(item, "id"));
        }
    } else if items.is_empty() {
        if let Some(context) = &inferred {
            println!("No compatible installed documentation matched {query:?} in this project.");
            for warning in project_warnings(context) {
                println!("  {}", terminal::safe_text(warning));
            }
            println!("Add --stack all to search every installed docset.");
        } else {
            println!("No installed documentation matched {query:?}.");
            println!("Try a broader term, or remove the docset and version filters.");
        }
        println!(
            "Run devline docs list to see available packs and devline docs install <id> to add one."
        );
    } else {
        if let Some(context) = &inferred {
            progress(
                cli,
                format_args!(
                    "Using project documentation. Run devline project to review versions."
                ),
            );
            for warning in project_warnings(context) {
                progress(cli, format_args!("  {}", terminal::safe_text(warning)));
            }
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
fn show_project_operation(cli: &Cli, action: &str, data: Value) -> Result<i32> {
    if cli.json {
        println!("{}", json!({"schema_version":1,"data":data}));
    } else if action == "job-started" {
        render_job(&data);
        println!(
            "Follow with devline jobs status {} --follow.",
            s(&data, "id")
        );
    } else {
        let status = if data["ready_offline"] == true {
            "Ready offline"
        } else {
            "Needs preparation"
        };
        println!(
            "Project  {}\n{status}\n{}",
            terminal::safe_text(s(&data, "path")),
            terminal::safe_text(s(&data, "detail"))
        );
    }
    Ok(if data["ready_offline"] == false { 3 } else { 0 })
}

fn run_project(cli: &Cli, path: &std::path::Path, arguments: &[std::ffi::OsString]) -> Result<i32> {
    if cli.json {
        bail!("Project commands own their standard output. Remove --json for project run.");
    }
    let runner = request(cli, json!({"command":"project.runner","path":path}))?;
    let mut command = if cli.offline {
        let mut command = Command::new("/usr/bin/sandbox-exec");
        command.args(["-p", "(version 1)(allow default)(deny network-outbound)(allow network-outbound (remote ip \"localhost:*\"))", "/bin/sh"]);
        command
    } else {
        Command::new("/bin/sh")
    };
    let status = command
        .arg(s(&runner, "adapter"))
        .arg("run")
        .arg(s(&runner, "path"))
        .arg(s(&runner, "cache"))
        .arg(s(&runner, "toolchain_path"))
        .args(arguments)
        .current_dir(s(&runner, "path"))
        .status()
        .context("Could not run the prepared project command")?;
    Ok(status.code().unwrap_or(1))
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
        Some(Commands::Project { path, command }) => {
            let path = path.clone().map_or_else(std::env::current_dir, Ok)?;
            let payload = match command {
                None | Some(Project::Status) => json!({"command":"project.context","path":path}),
                Some(Project::Readiness) => json!({"command":"project.readiness","path":path}),
                Some(Project::Prepare { background }) => {
                    let (action, data) = run_operation(
                        cli,
                        "project-prepared",
                        "project_prepare",
                        *background,
                        json!({"command":"project.prepare","path":path,"offline":cli.offline}),
                    )?;
                    return show_project_operation(cli, action, data);
                }
                Some(Project::Run { command }) => return run_project(cli, &path, command),
                Some(Project::Pin { id }) => {
                    let (pack, version) = pack_reference(id, true)?;
                    json!({"command":"project.pin","path":path,"id":pack,"version":version})
                }
                Some(Project::Unpin { id }) => {
                    json!({"command":"project.unpin","path":path,"id":id})
                }
            };
            let mut project = request(cli, payload)?;
            if matches!(command, Some(Project::Readiness)) {
                return show_project_operation(cli, "project-readiness", project);
            }
            let project_path = project["path"]
                .as_str()
                .map(std::path::Path::new)
                .unwrap_or(&path);
            project["infrastructure"] = cli_infra::project_config(cli, project_path)
                .unwrap_or_else(|error| json!({"error":format!("{error:#}")}));
            ("project", project)
        }
        Some(Commands::Infra { command }) => cli_infra::run(cli, command.as_ref())?,
        Some(Commands::Toolbox(options)) => cli_infra::toolbox(cli, options)?,
        Some(Commands::Offline { command }) => match command {
            None => (
                "offline",
                request(cli, json!({"command":"offline.status"}))?,
            ),
            Some(Offline::Status { path, selection }) => (
                "offline",
                request(
                    cli,
                    json!({"command":"offline.status","path":path,"docs":selection.docs,"starters":selection.starters}),
                )?,
            ),
            Some(Offline::Check { id, background }) => run_operation(
                cli,
                "verified",
                "verify",
                *background,
                json!({"command":"offline.check","id":id}),
            )?,
        },
        Some(Commands::Bundle { command }) => match command {
            Bundle::Export {
                path,
                selection,
                background,
            } => run_operation(
                cli,
                "bundle-exported",
                "bundle_export",
                *background,
                json!({"command":"bundle.export","path":path,"docs":selection.docs,"starters":selection.starters}),
            )?,
            Bundle::Inspect { path } => (
                "bundle",
                request(cli, json!({"command":"bundle.inspect","path":path}))?,
            ),
            Bundle::Import {
                path,
                trust_starters,
                background,
            } => run_operation(
                cli,
                "bundle-imported",
                "bundle_import",
                *background,
                json!({"command":"bundle.import","path":path,"trust_starters":trust_starters}),
            )?,
        },
        Some(Commands::Jobs { command }) => match command {
            None | Some(Jobs::List) => ("jobs", request(cli, json!({"command":"jobs.list"}))?),
            Some(Jobs::Status {
                id,
                offset,
                follow: true,
            }) => return watch_job(cli, id, *offset),
            Some(Jobs::Status {
                id,
                offset,
                follow: false,
            }) => (
                "job",
                request(
                    cli,
                    json!({"command":"jobs.status","job_id":id,"offset":offset}),
                )?,
            ),
            Some(Jobs::Cancel { id }) => (
                "job",
                request(cli, json!({"command":"jobs.cancel","job_id":id}))?,
            ),
        },
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
            Some(Docs::Install {
                id,
                version,
                background,
            }) => {
                let (id, pinned) = pack_reference(id, false)?;
                if pinned.zip(version.as_deref()).is_some_and(|(a, b)| a != b) {
                    bail!("The documentation reference and --version specify different releases.");
                }
                let version = pinned.or(version.as_deref());
                progress(cli, format_args!("Installing documentation for {id}…"));
                run_operation(
                    cli,
                    "install",
                    "docs_prepare",
                    *background,
                    json!({"command":"docs.install","id":id,"version":version}),
                )?
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
            Some(Docs::Updates { command }) => {
                let payload = match command {
                    None | Some(DocsUpdates::Status) => {
                        json!({"command":"docs.distribution.status"})
                    }
                    Some(DocsUpdates::Check) => {
                        if cli.offline {
                            bail!("Checking the documentation channel requires a connection.");
                        }
                        progress(
                            cli,
                            format_args!("Checking the documentation release channel…"),
                        );
                        json!({"command":"docs.distribution.check"})
                    }
                    Some(DocsUpdates::Configure { url, clear }) => {
                        let url = if *clear {
                            ""
                        } else {
                            url.as_deref().unwrap_or_default().trim()
                        };
                        if !*clear && url.is_empty() {
                            bail!("Enter a public HTTPS manifest URL or use --clear.");
                        }
                        json!({"command":"docs.distribution.configure","url":url})
                    }
                };
                ("doc-updates", request(cli, payload)?)
            }
            Some(Docs::Fetch { id, background }) => {
                if cli.offline {
                    bail!("Fetching documentation requires a connection. Use docs install for bundled packs.");
                }
                let (id, version) = pack_reference(id, false)?;
                progress(cli, format_args!("Fetching {id}…"));
                run_operation(
                    cli,
                    "doc-fetched",
                    "docs_install",
                    *background,
                    json!({"command":"docs.distribution.install","id":id,"version":version}),
                )?
            }
            Some(Docs::Rollback { id }) => {
                let (id, version) = pack_reference(id, false)?;
                (
                    "doc-rollback",
                    request(
                        cli,
                        json!({"command":"docs.distribution.rollback","id":id,"version":version}),
                    )?,
                )
            }
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
            Templates::Build { id, background, .. } => {
                if cli.offline {
                    bail!("Building with latest upstream dependencies requires a connection. Run this command while connected.");
                }
                progress(cli, format_args!("Refreshing dependencies for {id}, then testing with network access blocked…"));
                run_operation(
                    cli,
                    "build",
                    "build",
                    *background,
                    json!({"command":"templates.build","id":id}),
                )?
            }
            Templates::Inspect { id } => (
                "template-detail",
                request(cli, json!({"command":"templates.inspect","id":id}))?,
            ),
            Templates::Verify { id, background } => run_operation(
                cli,
                "verified",
                "verify",
                *background,
                json!({"command":"offline.check","id":id}),
            )?,
            Templates::Remove { id } => {
                pack_reference(id, true)?;
                (
                    "template-removed",
                    request(cli, json!({"command":"templates.remove","id":id}))?,
                )
            }
        },
        Some(Commands::Prepare { id, background }) => {
            progress(
                cli,
                format_args!("Installing starter and matching documentation for {id}…"),
            );
            run_operation(
                cli,
                "prepare",
                "prepare",
                *background,
                json!({"command":"prepare","id":id}),
            )?
        }
        Some(Commands::Doctor { id }) => {
            ("doctor", request(cli, json!({"command":"doctor","id":id}))?)
        }
        Some(Commands::New {
            id,
            destination,
            dry_run,
            services,
            ports,
        }) => {
            let selected = match id {
                Some(id) => id.clone(),
                None => {
                    let list = request(cli, json!({"command":"templates.list"}))?;
                    let items: Vec<_> =
                        list.as_array().context("Unexpected list")?.iter().collect();
                    if items.is_empty() {
                        bail!("No starters are available. Browse the current catalog with devline templates list.");
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
                    json!({"command":"new.ready","id":selected,"destination":path,"dry_run":dry_run,"offline":cli.offline,"services":services,"ports":cli_infra::port_overrides(ports)?}),
                )?,
            )
        }
    };
    let ready = match action {
        "doctor" => data["ready"].as_bool(),
        "prepare" => data["doctor"]["ready"].as_bool(),
        "offline" | "verified" | "toolbox" => data["ready_offline"].as_bool(),
        "infra-running" => data["running"].as_bool(),
        "new" if data["dry_run"] == true => data["can_create"].as_bool(),
        "bundle" => data["compatible"].as_bool(),
        _ => None,
    };
    if cli.json {
        println!("{}", json!({"schema_version":1,"data":data}));
    } else {
        match action {
            "project" => render_project(&data),
            "offline" => render_offline(&data),
            action if action == "toolbox" || action.starts_with("infra-") => {
                cli_infra::render(action, &data)
            }
            "verified" => {
                let template = &data["template"];
                println!(
                    "Verified {}@{} with external networking denied.",
                    s(template, "id"),
                    s(template, "version")
                );
            }
            "bundle" => render_bundle(&data),
            "bundle-exported" => println!(
                "Exported {} docsets and {} starters to {}.",
                data["docs"],
                data["starters"],
                s(&data, "path")
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
                render_job(&data);
                if let Some(log) = data["log"].as_str() {
                    print!("{}", terminal::safe_text(log));
                }
            }
            "job-started" => {
                render_job(&data);
                println!(
                    "Follow with devline jobs status {} --follow.",
                    s(&data, "id")
                );
            }
            "doc-updates" => render_doc_updates(&data),
            "doc-fetched" => println!(
                "Installed and activated {}@{} · {} articles.",
                s(&data, "id"),
                s(&data, "version"),
                data["count"]
            ),
            "doc-rollback" => println!(
                "Active documentation is {}@{}.",
                s(&data, "id"),
                s(&data, "active_version")
            ),
            "template-detail" => render_starter_details(&data),
            "template-removed" => println!("Removed the installed starter release."),
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
                        s(&data, "test_command")
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
    if action == "infra-update-batch" && data["failed"].as_u64().unwrap_or(0) > 0 {
        return Ok(1);
    }
    if ready == Some(false) {
        return Ok(3);
    }
    Ok(0)
}
fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

fn job_worker() -> Option<Result<()>> {
    let mut args = std::env::args_os().skip(1);
    if args.next()? != "__job-worker" {
        return None;
    }
    Some((|| {
        let id = args.next().context("Missing job ID")?;
        let home = PathBuf::from(args.next().context("Missing job library path")?);
        let catalog = PathBuf::from(args.next().context("Missing job catalog path")?);
        if args.next().is_some() {
            bail!("Unexpected job worker argument");
        }
        devkit_core::tasks::run_worker(home, catalog, id.to_str().context("Invalid job ID")?)
    })())
}

fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    if let Some(result) = job_worker() {
        if let Err(error) = result {
            eprintln!("devline worker: {error:#}");
            std::process::exit(1);
        }
        return;
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
