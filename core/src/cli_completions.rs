//! Shell completion scripts and live catalog candidates.

use crate::{cli_infra, request, s, Cli};
use anyhow::{Context, Result};
use clap::{CommandFactory, ValueEnum};
use serde_json::json;
use std::{collections::BTreeSet, io};

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

pub(super) fn complete(cli: &Cli) -> Option<i32> {
    let value = ["_DEVLINE_ZSH_COMPLETION", "_DEVKIT_ZSH_COMPLETION"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))?;
    let kind = CompletionKind::from_str(&value, false).ok()?;
    Some(if completion_values(cli, kind).is_ok() {
        0
    } else {
        1
    })
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

pub(super) fn print_completions(shell: clap_complete::Shell) -> Result<()> {
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
    candidates=("${{(@f)$(_DEVLINE_ZSH_COMPLETION="$kind" command devline "${{catalog_args[@]}}" 2>/dev/null)}}")
    [[ -n "${{candidates[1]-}}" ]] && compadd "$@" -a candidates
}}
{body}"#
    );
    Ok(())
}
