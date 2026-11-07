use super::version::{matches, numeric, Grammar, Rule};
use anyhow::{Context, Result};
use roxmltree::Node;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SWIFT_SOURCE_BYTES: u64 = 64 * 1024;
const MAX_SWIFT_SOURCE_ENTRIES: usize = 256;
const MARKERS: &[(&str, &[&str])] = &[
    ("node", &["package.json", ".node-version", ".nvmrc"]),
    ("swift", &["Package.swift", ".swift-version"]),
    ("go", &["go.mod", "go.work"]),
    (
        "rust",
        &["Cargo.toml", "rust-toolchain", "rust-toolchain.toml"],
    ),
    (
        "python",
        &[
            "pyproject.toml",
            "requirements.txt",
            ".python-version",
            "Pipfile",
        ],
    ),
    ("ruby", &["Gemfile", ".ruby-version"]),
    (
        "java",
        &[
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            ".java-version",
        ],
    ),
    ("elixir", &["mix.exs"]),
    ("dotnet", &["global.json"]),
];

#[derive(Clone, Debug, Serialize)]
pub(super) struct Dependency {
    pub id: &'static str,
    pub version: Option<String>,
    pub requirement: Option<String>,
    pub source: String,
    #[serde(skip)]
    pub rule: Rule,
}

pub(super) struct Detected {
    pub path: PathBuf,
    pub stacks: Vec<&'static str>,
    pub dependencies: Vec<Dependency>,
}

impl Detected {
    pub fn stack(&self) -> Option<&'static str> {
        (self.stacks.len() == 1).then(|| self.stacks[0])
    }
}

fn read(path: &Path) -> Option<String> {
    read_bounded(path, MAX_MANIFEST_BYTES)
}

fn read_bounded(path: &Path, limit: u64) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > limit {
        return None;
    }
    let mut value = String::new();
    File::open(path)
        .ok()?
        .take(limit + 1)
        .read_to_string(&mut value)
        .ok()?;
    (value.len() as u64 <= limit).then_some(value)
}

fn toml(root: &Path, file: &str) -> Option<toml::Value> {
    toml::from_str(&read(&root.join(file))?).ok()
}

fn toml_text<'a>(value: &'a toml::Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .try_fold(value, |value, key| value.get(key))?
        .as_str()
}

fn trimmed(text: &str) -> Option<&str> {
    let text = text.trim();
    (!text.is_empty()).then_some(text)
}

fn quoted(text: &str) -> Option<(&str, &str)> {
    let start = text.find(['"', '\''])?;
    let quote = text.as_bytes()[start] as char;
    let body = &text[start + 1..];
    let end = body.find(quote)?;
    Some((&body[..end], &body[end + 1..]))
}

fn uncomment(line: &str, marker: char) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (offset, ch) in line.char_indices() {
        if escaped {
            escaped = false;
        } else if quote.is_some() && ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '\'' | '"') {
            quote = Some(ch);
        } else if quote.is_none() && ch == marker {
            return &line[..offset];
        }
    }
    line
}

fn runtime_file(root: &Path, file: &str) -> Option<String> {
    let value = read(&root.join(file))?;
    let line = value
        .lines()
        .map(|line| uncomment(line, '#').trim())
        .find(|line| !line.is_empty() && !line.starts_with('#'))?;
    Some(line.to_owned())
}

fn tool_version(root: &Path, name: &str) -> Option<String> {
    let contents = read(&root.join(".tool-versions"))?;
    contents.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        (fields.next()? == name).then(|| fields.next().map(str::to_owned))?
    })
}

fn exact(raw: &str) -> Rule {
    Rule::Exact(raw.trim().trim_start_matches('v').to_owned())
}

fn dependency(
    id: &'static str,
    source: impl Into<String>,
    version: Option<String>,
    requirement: Option<String>,
    rule: Rule,
) -> Dependency {
    Dependency {
        id,
        source: source.into(),
        version,
        requirement,
        rule,
    }
}

fn pin(id: &'static str, raw: &str, source: &str) -> Dependency {
    let raw = raw.trim().trim_start_matches('v');
    dependency(id, source, Some(raw.to_owned()), None, exact(raw))
}

fn runtime(id: &'static str, raw: &str, source: &str, width: usize) -> Dependency {
    let raw = raw.trim().trim_start_matches('v');
    if numeric(raw) {
        dependency(
            id,
            source,
            Some(raw.to_owned()),
            None,
            Rule::Series(vec![raw.to_owned()], width),
        )
    } else {
        dependency(id, source, None, Some(raw.to_owned()), Rule::Unknown)
    }
}

fn requirement(id: &'static str, raw: &str, source: &str, grammar: Grammar) -> Dependency {
    let raw = raw.trim();
    let version = raw
        .strip_prefix("==")
        .or_else(|| raw.strip_prefix('='))
        .map(str::trim)
        .filter(|value| numeric(value));
    if let Some(version) = version {
        return dependency(
            id,
            source,
            Some(version.to_owned()),
            Some(raw.to_owned()),
            exact(version),
        );
    }
    dependency(
        id,
        source,
        None,
        Some(raw.to_owned()),
        Rule::Constraint(raw.to_owned(), grammar),
    )
}

fn unknown(id: &'static str, source: &str, detail: &str) -> Dependency {
    dependency(id, source, None, Some(detail.to_owned()), Rule::Unknown)
}

fn unversioned(id: &'static str, source: &str) -> Dependency {
    dependency(id, source, None, None, Rule::Any)
}

fn lock<'a>(
    id: &'static str,
    versions: impl IntoIterator<Item = &'a str>,
    declared: Option<Dependency>,
    source: &str,
) -> Option<Dependency> {
    let mut versions: BTreeSet<_> = versions
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect();
    if versions.len() > 1 {
        if let Some(declared) = &declared {
            let filtered: BTreeSet<_> = versions
                .iter()
                .copied()
                .filter(|value| matches(&declared.rule, value) == Some(true))
                .collect();
            if !filtered.is_empty() {
                versions = filtered;
            }
        }
    }
    match versions.len() {
        0 => declared,
        1 => {
            let mut resolved = pin(id, versions.first().unwrap(), source);
            resolved.requirement = declared.and_then(|item| item.requirement);
            Some(resolved)
        }
        _ => Some(unknown(
            id,
            source,
            &format!(
                "multiple locked versions: {}",
                versions.into_iter().collect::<Vec<_>>().join(", ")
            ),
        )),
    }
}

fn lock_packages<'a>(value: &'a toml::Value, package: &'a str) -> impl Iterator<Item = &'a str> {
    value
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter(move |row| toml_text(row, &["name"]) == Some(package))
        .filter_map(|row| toml_text(row, &["version"]))
}

fn python_requirement(line: &str) -> Option<Dependency> {
    let line = uncomment(line, '#').trim();
    let (name, rest) = line.split_at(
        line.find(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
            .unwrap_or(line.len()),
    );
    if !name.eq_ignore_ascii_case("fastapi") {
        return None;
    }
    let rest = rest.trim();
    let rest = if let Some(extras) = rest.strip_prefix('[') {
        extras.split_once(']')?.1.trim()
    } else {
        rest
    };
    let (value, marker) = rest
        .split_once(';')
        .map_or((rest, None), |(value, marker)| (value, trimmed(marker)));
    let value = value.split(" --").next().unwrap_or_default().trim();
    let item = if marker.is_some() {
        unknown("fastapi", "", rest)
    } else if value.is_empty() {
        unversioned("fastapi", "")
    } else {
        requirement("fastapi", value, "", Grammar::Constraint)
    };
    Some(item)
}

fn sourced(mut item: Dependency, source: &str) -> Dependency {
    item.source = source.to_owned();
    item
}

fn python(root: &Path, output: &mut Vec<Dependency>) {
    let manifest = toml(root, "pyproject.toml");
    let legacy = manifest
        .as_ref()
        .and_then(|value| value.get("tool")?.get("poetry")?.get("dependencies"));
    let requires_python = manifest
        .as_ref()
        .and_then(|value| toml_text(value, &["project", "requires-python"]))
        .or_else(|| legacy.and_then(|value| toml_text(value, &["python"])));
    let python = runtime_file(root, ".python-version")
        .map(|value| runtime("python", &value, ".python-version", 2))
        .or_else(|| {
            tool_version(root, "python").map(|value| runtime("python", &value, ".tool-versions", 2))
        })
        .or_else(|| {
            requires_python
                .map(|value| requirement("python", value, "pyproject.toml", Grammar::Constraint))
        })
        .or_else(|| {
            (manifest.is_none() && root.join("pyproject.toml").is_file()).then(|| {
                unknown(
                    "python",
                    "pyproject.toml",
                    "an unreadable or invalid manifest",
                )
            })
        })
        .unwrap_or_else(|| unversioned("python", "Python project"));
    output.push(python);
    let mut fastapi = manifest
        .as_ref()
        .and_then(|value| value.get("project")?.get("dependencies")?.as_array())
        .and_then(|items| {
            items
                .iter()
                .filter_map(toml::Value::as_str)
                .find_map(python_requirement)
        })
        .map(|item| sourced(item, "pyproject.toml"))
        .or_else(|| {
            let item = legacy?.get("fastapi")?;
            Some(
                item.as_str()
                    .or_else(|| toml_text(item, &["version"]))
                    .map_or_else(
                        || unknown("fastapi", "pyproject.toml", "path or Git dependency"),
                        |value| {
                            requirement("fastapi", value, "pyproject.toml", Grammar::Constraint)
                        },
                    ),
            )
        });
    for file in ["requirements.txt", "requirements.lock"] {
        if let Some(item) =
            read(&root.join(file)).and_then(|body| body.lines().find_map(python_requirement))
        {
            if fastapi.is_none() || item.version.is_some() {
                fastapi = Some(sourced(item, file));
            }
        }
    }
    for file in ["uv.lock", "poetry.lock"] {
        if let Some(value) = toml(root, file) {
            let versions: Vec<_> = lock_packages(&value, "fastapi").collect();
            if !versions.is_empty() {
                fastapi = lock("fastapi", versions, fastapi, file);
                break;
            }
        }
    }
    if let Some(value) =
        read(&root.join("Pipfile.lock")).and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
    {
        if let Some(raw) = value["default"]["fastapi"]["version"]
            .as_str()
            .and_then(|value| value.strip_prefix("=="))
        {
            if fastapi
                .as_ref()
                .is_none_or(|item| !matches!(item.source.as_str(), "uv.lock" | "poetry.lock"))
            {
                fastapi = lock("fastapi", [raw], fastapi, "Pipfile.lock");
            }
        }
    }
    output.extend(fastapi);
}

fn rust(root: &Path, output: &mut Vec<Dependency>) {
    let manifest = toml(root, "Cargo.toml");
    let workspace = root.ancestors().skip(1).take(12).find_map(|ancestor| {
        let manifest = toml(ancestor, "Cargo.toml")?;
        manifest.get("workspace")?;
        Some((ancestor, manifest))
    });
    let toolchain = toml(root, "rust-toolchain.toml")
        .and_then(|value| toml_text(&value, &["toolchain", "channel"]).map(str::to_owned));
    let inherited_minimum = workspace
        .as_ref()
        .and_then(|(_, value)| toml_text(value, &["workspace", "package", "rust-version"]));
    let minimum = manifest
        .as_ref()
        .and_then(|value| toml_text(value, &["package", "rust-version"]))
        .or(inherited_minimum);
    output.push(
        toolchain
            .map(|value| runtime("rust", &value, "rust-toolchain.toml", 2))
            .or_else(|| {
                runtime_file(root, "rust-toolchain")
                    .map(|value| runtime("rust", &value, "rust-toolchain", 2))
            })
            .or_else(|| {
                tool_version(root, "rust").map(|value| runtime("rust", &value, ".tool-versions", 2))
            })
            .or_else(|| minimum.map(|value| runtime("rust", value, "Cargo.toml", 2)))
            .or_else(|| {
                (manifest.is_none() && root.join("Cargo.toml").is_file())
                    .then(|| unknown("rust", "Cargo.toml", "an unreadable or invalid manifest"))
            })
            .unwrap_or_else(|| unversioned("rust", "Cargo.toml")),
    );
    let declared = manifest.as_ref().and_then(|manifest| {
        manifest
            .get("dependencies")
            .or_else(|| manifest.get("workspace")?.get("dependencies"))?
            .as_table()
    });
    let inherited = workspace
        .as_ref()
        .and_then(|(_, manifest)| manifest.get("workspace")?.get("dependencies"));
    let lockfile = toml(root, "Cargo.lock").or_else(|| {
        workspace
            .as_ref()
            .and_then(|(ancestor, _)| toml(ancestor, "Cargo.lock"))
    });
    for package in ["axum", "tauri"] {
        let declared = declared.and_then(|items| {
            items.iter().find_map(|(name, value)| {
                let resolved =
                    if value.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                        inherited
                            .and_then(|entries| entries.get(name))
                            .unwrap_or(value)
                    } else {
                        value
                    };
                (name == package || toml_text(resolved, &["package"]) == Some(package))
                    .then_some(resolved)
            })
        });
        let declared = declared.map(|item| {
            item.as_str()
                .or_else(|| toml_text(item, &["version"]))
                .map_or_else(
                    || unknown(package, "Cargo.toml", "path or Git dependency"),
                    |value| requirement(package, value, "Cargo.toml", Grammar::Cargo),
                )
        });
        let versions = lockfile
            .iter()
            .flat_map(|value| lock_packages(value, package));
        output.extend(lock(package, versions, declared, "Cargo.lock"));
    }
}

fn swift_imports_ui(source: &str) -> bool {
    let mut block_depth = 0usize;
    let mut multiline_string = false;
    for line in source.lines() {
        let mut line = line.trim_start();
        while block_depth > 0 || line.starts_with("/*") {
            let open = line.find("/*");
            let close = line.find("*/");
            if let Some(start) = open.filter(|start| close.is_none_or(|end| *start < end)) {
                block_depth += 1;
                line = &line[start + 2..];
            } else if let Some(end) = close {
                block_depth -= 1;
                line = line[end + 2..].trim_start();
            } else {
                break;
            }
        }
        if block_depth > 0 || line.starts_with("//") {
            continue;
        }
        if line.matches("\"\"\"").count() % 2 == 1 {
            multiline_string = !multiline_string;
            continue;
        }
        if multiline_string {
            continue;
        }
        let mut tokens = line.split_whitespace().skip_while(|part| {
            part.starts_with('@')
                || matches!(
                    *part,
                    "private" | "fileprivate" | "internal" | "package" | "public"
                )
        });
        if tokens.next() != Some("import") {
            continue;
        }
        let module = match tokens.next() {
            Some(
                "typealias" | "struct" | "class" | "enum" | "protocol" | "let" | "var" | "func",
            ) => tokens.next(),
            name => name,
        };
        if module.and_then(|name| name.split(['.', ';']).next()) == Some("SwiftUI") {
            return true;
        }
    }
    false
}

fn has_swiftui_source(root: &Path) -> bool {
    let sources = root.join("Sources");
    if !fs::symlink_metadata(&sources).is_ok_and(|metadata| metadata.is_dir()) {
        return false;
    }
    let mut pending = vec![(sources, 0)];
    let mut budget = MAX_SWIFT_SOURCE_ENTRIES;
    while let Some((directory, depth)) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.take(budget).flatten() {
            budget -= 1;
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() && depth < 5 && !entry.file_name().to_string_lossy().starts_with('.') {
                pending.push((path, depth + 1));
            } else if kind.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "swift")
                && read_bounded(&path, MAX_SWIFT_SOURCE_BYTES)
                    .is_some_and(|source| swift_imports_ui(&source))
            {
                return true;
            }
        }
        if budget == 0 {
            break;
        }
    }
    false
}

fn swift(root: &Path, output: &mut Vec<Dependency>) {
    let tools_version =
        read_bounded(&root.join("Package.swift"), MAX_SWIFT_SOURCE_BYTES).and_then(|body| {
            let version = body.lines().take(8).find_map(|line| {
                line.trim_start()
                    .strip_prefix("//")?
                    .trim_start()
                    .strip_prefix("swift-tools-version:")?
                    .split_whitespace()
                    .next()
                    .filter(|value| numeric(value))
            })?;
            Some(requirement(
                "swift",
                &format!(">={version}"),
                "Package.swift",
                Grammar::Constraint,
            ))
        });
    output.push(
        runtime_file(root, ".swift-version")
            .map(|value| runtime("swift", &value, ".swift-version", 2))
            .or_else(|| {
                tool_version(root, "swift")
                    .map(|value| runtime("swift", &value, ".tool-versions", 2))
            })
            .or(tools_version)
            .unwrap_or_else(|| unversioned("swift", "Package.swift")),
    );
    if has_swiftui_source(root) {
        output.push(unversioned("swiftui", "Sources"));
    }
}

fn go(root: &Path, output: &mut Vec<Dependency>) {
    let (file, body) = read(&root.join("go.mod"))
        .map(|body| ("go.mod", body))
        .or_else(|| read(&root.join("go.work")).map(|body| ("go.work", body)))
        .unwrap_or(("go.mod", String::new()));
    let lines: Vec<_> = body
        .lines()
        .map(|line| line.split("//").next().unwrap_or_default().trim())
        .collect();
    let toolchain = lines.iter().find_map(|line| {
        let mut words = line.split_whitespace();
        (words.next()? == "toolchain").then(|| words.next()?.strip_prefix("go"))?
    });
    let minimum = lines.iter().find_map(|line| {
        let mut words = line.split_whitespace();
        (words.next()? == "go").then(|| words.next())?
    });
    output.push(
        toolchain
            .or(minimum)
            .map(|version| runtime("go", version, file, 2))
            .or_else(|| {
                tool_version(root, "go").map(|version| runtime("go", &version, ".tool-versions", 2))
            })
            .unwrap_or_else(|| unversioned("go", file)),
    );
    let chi = |module: &str| {
        module == "github.com/go-chi/chi" || module.starts_with("github.com/go-chi/chi/v")
    };
    let mut block = "";
    let mut selected = None;
    let mut replacement = None;
    for line in lines {
        let words: Vec<_> = line.split_whitespace().collect();
        match words.as_slice() {
            ["require" | "replace", "("] => block = words[0],
            [")"] => block = "",
            [] => {}
            _ => {
                let (kind, args) = if matches!(words[0], "require" | "replace") {
                    (words[0], &words[1..])
                } else {
                    (block, words.as_slice())
                };
                if args.first().is_none_or(|module| !chi(module)) {
                    continue;
                }
                if kind == "require" {
                    selected = args.get(1).map(|value| pin("chi", value, file));
                } else if kind == "replace" {
                    if let Some((_, target)) = line.split_once("=>") {
                        let mut target = target.split_whitespace();
                        replacement = Some(match (target.next(), target.next()) {
                            (Some(module), Some(version)) if chi(module) => {
                                pin("chi", version, file)
                            }
                            _ => unknown("chi", file, "replaced with a local or alternate module"),
                        });
                    }
                }
            }
        }
    }
    output.extend(replacement.or(selected));
}

fn ruby(root: &Path, output: &mut Vec<Dependency>) {
    let gemfile = read(&root.join("Gemfile")).unwrap_or_default();
    let lines: Vec<_> = gemfile
        .lines()
        .map(|line| uncomment(line, '#').trim())
        .collect();
    let declared_ruby = lines
        .iter()
        .find_map(|line| quoted(line.strip_prefix("ruby ")?.trim()).map(|item| item.0));
    output.push(
        runtime_file(root, ".ruby-version")
            .map(|value| {
                runtime(
                    "ruby",
                    value.trim_start_matches("ruby-"),
                    ".ruby-version",
                    2,
                )
            })
            .or_else(|| {
                tool_version(root, "ruby").map(|value| runtime("ruby", &value, ".tool-versions", 2))
            })
            .or_else(|| {
                declared_ruby.map(|value| {
                    if numeric(value) {
                        runtime("ruby", value, "Gemfile", 2)
                    } else {
                        requirement("ruby", value, "Gemfile", Grammar::Constraint)
                    }
                })
            })
            .unwrap_or_else(|| unversioned("ruby", "Gemfile")),
    );
    let declared = lines.iter().find_map(|line| {
        let (name, tail) = quoted(line.strip_prefix("gem")?)?;
        if name != "rails" {
            return None;
        }
        let mut values = Vec::new();
        let mut tail = tail.trim();
        while let Some(value) = tail
            .strip_prefix(',')
            .map(str::trim_start)
            .filter(|value| value.starts_with(['"', '\'']))
        {
            let (value, rest) = quoted(value)?;
            values.push(value);
            tail = rest.trim();
        }
        Some(if values.is_empty() {
            if tail.contains("git:") || tail.contains("github:") || tail.contains("path:") {
                unknown("rails", "Gemfile", "path or Git dependency")
            } else {
                unversioned("rails", "Gemfile")
            }
        } else {
            requirement("rails", &values.join(","), "Gemfile", Grammar::Constraint)
        })
    });
    let lockfile = read(&root.join("Gemfile.lock")).unwrap_or_default();
    let versions = lockfile.lines().filter_map(|line| {
        line.strip_prefix("    rails (")?
            .split_once(')')
            .map(|item| item.0)
    });
    output.extend(lock("rails", versions, declared, "Gemfile.lock"));
}

fn elixir(root: &Path, output: &mut Vec<Dependency>) {
    let manifest = read(&root.join("mix.exs")).unwrap_or_default();
    let body = manifest
        .lines()
        .map(|line| uncomment(line, '#'))
        .collect::<Vec<_>>()
        .join("\n");
    let declared = body.split_once("elixir:").and_then(|(_, tail)| {
        let tail = tail.trim_start();
        tail.starts_with(['"', '\''])
            .then(|| quoted(tail).map(|item| item.0))
            .flatten()
    });
    output.push(
        tool_version(root, "elixir")
            .map(|value| {
                let value = value
                    .split_once("-otp-")
                    .map_or(value.as_str(), |(version, _)| version);
                runtime("elixir", value, ".tool-versions", 2)
            })
            .or_else(|| {
                declared.map(|value| requirement("elixir", value, "mix.exs", Grammar::Constraint))
            })
            .unwrap_or_else(|| unversioned("elixir", "mix.exs")),
    );
    let lockfile = read(&root.join("mix.lock")).unwrap_or_default();
    for name in ["phoenix", "ecto"] {
        let spec = format!(":{}", name);
        let declared = body.match_indices(&spec).find_map(|(offset, _)| {
            let rest = &body[offset + spec.len()..];
            let rest = rest.trim_start().strip_prefix(',')?.trim_start();
            Some(if rest.starts_with(['"', '\'']) {
                requirement(name, quoted(rest)?.0, "mix.exs", Grammar::Constraint)
            } else {
                unknown(name, "mix.exs", "path, Git, or computed dependency")
            })
        });
        let key = format!("\"{name}\"");
        let versions = lockfile.lines().filter_map(|line| {
            let (_, body) = line.split_once(&key)?;
            let (_, body) = body.split_once("{:hex,")?;
            let (_, value) = body.split_once(',')?;
            quoted(value).map(|item| item.0)
        });
        output.extend(lock(name, versions, declared, "mix.lock"));
    }
}

fn xml_child<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.children().find(|child| child.has_tag_name(name))
}

fn xml_text<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    trimmed(xml_child(node, name)?.text()?)
}

fn maven_value<'a>(node: Node<'a, '_>, mut value: &'a str) -> Option<&'a str> {
    for _ in 0..6 {
        let Some(key) = value
            .strip_prefix("${")
            .and_then(|value| value.strip_suffix('}'))
        else {
            return Some(value);
        };
        value = xml_text(xml_child(node, "properties")?, key)?;
    }
    None
}

fn java_version(raw: &str) -> Option<&str> {
    raw.split('-')
        .find(|part| numeric(part))
        .map(|value| value.strip_prefix("1.").unwrap_or(value))
}

fn java(root: &Path, output: &mut Vec<Dependency>) {
    let pom = read(&root.join("pom.xml"));
    let parsed = pom
        .as_ref()
        .and_then(|raw| roxmltree::Document::parse(raw).ok());
    let project = parsed.as_ref().map(roxmltree::Document::root_element);
    let maven_runtime = project.and_then(|node| {
        let properties = xml_child(node, "properties")?;
        [
            "maven.compiler.release",
            "java.version",
            "maven.compiler.source",
        ]
        .into_iter()
        .find_map(|key| maven_value(node, xml_text(properties, key)?))
    });
    let maven_spring = project.and_then(|node| {
        let boot = |entry| xml_text(entry, "groupId") == Some("org.springframework.boot");
        let parent = xml_child(node, "parent").filter(|entry| boot(*entry));
        let managed = xml_child(node, "dependencyManagement")
            .and_then(|entry| xml_child(entry, "dependencies"));
        let dependencies = xml_child(node, "dependencies");
        let entry = [
            parent,
            managed.and_then(|items| items.children().find(|entry| boot(*entry))),
            dependencies.and_then(|items| items.children().find(|entry| boot(*entry))),
        ]
        .into_iter()
        .flatten()
        .next()?;
        Some(
            xml_text(entry, "version")
                .and_then(|value| maven_value(node, value))
                .map_or_else(
                    || {
                        unknown(
                            "spring-boot",
                            "pom.xml",
                            "inherited or computed Spring Boot version",
                        )
                    },
                    |value| {
                        if numeric(value) {
                            pin("spring-boot", value, "pom.xml")
                        } else {
                            requirement("spring-boot", value, "pom.xml", Grammar::Constraint)
                        }
                    },
                ),
        )
    });
    let mut gradle_runtime = None;
    let mut gradle_spring = None;
    for file in ["build.gradle.kts", "build.gradle"] {
        let Some(body) = read(&root.join(file)) else {
            continue;
        };
        for line in body.lines() {
            let line = line.split("//").next().unwrap_or_default();
            let direct = [
                "JavaLanguageVersion.of(",
                "jvmToolchain(",
                "JavaVersion.VERSION_",
            ]
            .into_iter()
            .find_map(|key| {
                let tail = line.split_once(key)?.1.trim_start();
                trimmed(tail.split(|ch: char| !ch.is_ascii_digit()).next()?)
            });
            if let Some(value) = direct {
                gradle_runtime = Some(runtime("java", value, file, 1));
            }
            if let Some((before, tail)) = line.split_once("org.springframework.boot") {
                let version = before
                    .split(|ch: char| !ch.is_ascii_alphanumeric())
                    .any(|token| token == "id")
                    .then(|| {
                        tail.split_once("version")
                            .and_then(|(_, value)| quoted(value))
                    })
                    .flatten()
                    .map(|(value, _)| value)
                    .or_else(|| {
                        tail.strip_prefix(':')?
                            .split(':')
                            .nth(1)?
                            .split(['"', '\''])
                            .next()
                            .filter(|value| numeric(value))
                    });
                if gradle_spring.is_none() || version.is_some() {
                    gradle_spring = Some(version.map_or_else(
                        || {
                            unknown(
                                "spring-boot",
                                file,
                                "inherited or computed Spring Boot version",
                            )
                        },
                        |value| pin("spring-boot", value, file),
                    ));
                }
            }
        }
    }
    let sdkman = read(&root.join(".sdkmanrc")).and_then(|body| {
        body.lines()
            .find_map(|line| line.strip_prefix("java=").map(str::to_owned))
    });
    let configured = runtime_file(root, ".java-version")
        .map(|value| (value, ".java-version"))
        .or_else(|| sdkman.map(|value| (value, ".sdkmanrc")))
        .or_else(|| tool_version(root, "java").map(|value| (value, ".tool-versions")));
    let configured = configured.as_ref().map(|(raw, file)| {
        java_version(raw).map_or_else(
            || unknown("java", file, raw),
            |value| runtime("java", value, file, 1),
        )
    });
    output.push(
        configured
            .or_else(|| {
                maven_runtime.map(|value| {
                    runtime("java", java_version(value).unwrap_or(value), "pom.xml", 1)
                })
            })
            .or(gradle_runtime)
            .or_else(|| {
                (parsed.is_none() && root.join("pom.xml").is_file())
                    .then(|| unknown("java", "pom.xml", "an unreadable or invalid manifest"))
            })
            .unwrap_or_else(|| unversioned("java", "Java project")),
    );
    output.extend(maven_spring.or(gradle_spring));
}

fn project_files(root: &Path) -> (Vec<PathBuf>, bool) {
    let mut files = Vec::new();
    let mut solution = false;
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("csproj" | "fsproj") if path.is_file() => files.push(path),
                Some("sln" | "slnx") if path.is_file() => solution = true,
                _ => {}
            }
        }
    }
    files.sort();
    files.truncate(32);
    (files, solution)
}

fn dotnet(root: &Path, mut files: Vec<PathBuf>, solution: bool, output: &mut Vec<Dependency>) {
    if files.is_empty() && solution {
        let mut directories: Vec<_> = ["src", "app"]
            .into_iter()
            .map(|name| root.join(name))
            .collect();
        if let Ok(children) = fs::read_dir(root) {
            let mut children: Vec<_> = children
                .flatten()
                .filter_map(|entry| {
                    (entry.file_type().ok()?.is_dir()
                        && !entry.file_name().to_string_lossy().starts_with('.'))
                    .then(|| entry.path())
                })
                .collect();
            children.sort();
            directories.extend(children.into_iter().take(20));
        }
        for directory in directories {
            let (direct, _) = project_files(&directory);
            files.extend(direct);
            if let Ok(children) = fs::read_dir(&directory) {
                for child in children
                    .flatten()
                    .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                    .take(20)
                {
                    files.extend(project_files(&child.path()).0);
                    if files.len() >= 32 {
                        break;
                    }
                }
            }
            if files.len() >= 32 {
                break;
            }
        }
        files.sort();
        files.dedup();
        files.truncate(32);
    }
    let mut target_frameworks = BTreeSet::new();
    let mut aspnet = false;
    let mut manifest = "global.json".to_owned();
    if let Some(path) = files.first() {
        manifest = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
    }
    files.push(root.join("Directory.Build.props"));
    for file in files {
        let Some(raw) = read(&file) else {
            continue;
        };
        let Ok(xml) = roxmltree::Document::parse(&raw) else {
            continue;
        };
        let node = xml.root_element();
        aspnet |= node
            .attribute("Sdk")
            .is_some_and(|sdk| sdk.contains("Microsoft.NET.Sdk.Web"));
        for entry in node.descendants().filter(Node::is_element) {
            match entry.tag_name().name() {
                "TargetFramework" | "TargetFrameworks" | "TargetFrameworkVersion" => {
                    for target in entry
                        .text()
                        .unwrap_or_default()
                        .split(';')
                        .filter_map(trimmed)
                    {
                        let target = target.strip_prefix("net").unwrap_or(target);
                        target_frameworks.insert(
                            target
                                .split('-')
                                .next()
                                .unwrap_or(target)
                                .trim_start_matches('v')
                                .to_owned(),
                        );
                    }
                }
                "FrameworkReference" | "PackageReference" => {
                    aspnet |= entry
                        .attribute("Include")
                        .is_some_and(|name| name.starts_with("Microsoft.AspNetCore."));
                }
                _ => {}
            }
        }
    }
    let sdk = read(&root.join("global.json"))
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|value| value["sdk"]["version"].as_str().map(str::to_owned));
    let detect = |id| {
        if target_frameworks.is_empty() {
            return sdk.as_deref().map_or_else(
                || unversioned(id, &manifest),
                |version| runtime(id, version, "global.json", 2),
            );
        }
        let values: Vec<_> = target_frameworks.iter().cloned().collect();
        let label = values.join(", ");
        if values.iter().all(|value| numeric(value)) {
            dependency(
                id,
                &manifest,
                (values.len() == 1).then(|| label.clone()),
                Some(label),
                Rule::Series(values, 2),
            )
        } else {
            unknown(id, &manifest, &label)
        }
    };
    output.push(detect("csharp"));
    if aspnet {
        output.push(detect("aspnet"));
    }
}

fn node(root: &Path, output: &mut Vec<Dependency>) {
    let manifest = read(&root.join("package.json"))
        .and_then(|body| serde_json::from_str::<Value>(&body).ok())
        .unwrap_or_default();
    let lock = read(&root.join("package-lock.json"))
        .and_then(|body| serde_json::from_str::<Value>(&body).ok())
        .unwrap_or_default();
    let runtime = [".node-version", ".nvmrc"]
        .into_iter()
        .find_map(|file| runtime_file(root, file).map(|version| runtime("node", &version, file, 1)))
        .or_else(|| {
            tool_version(root, "node").map(|version| runtime("node", &version, ".tool-versions", 1))
        })
        .or_else(|| {
            manifest["engines"]["node"].as_str().map(|constraint| {
                let normalized = constraint
                    .split("||")
                    .map(|range| range.split_whitespace().collect::<Vec<_>>().join(","))
                    .collect::<Vec<_>>()
                    .join("||");
                let mut requirement =
                    requirement("node", &normalized, "package.json", Grammar::Constraint);
                requirement.requirement = Some(constraint.to_owned());
                requirement
            })
        })
        .unwrap_or_else(|| unversioned("node", "package.json"));
    output.push(runtime);
    for (package, id) in [
        ("typescript", "typescript"),
        ("react", "react"),
        ("@playwright/test", "playwright"),
        ("vite", "vite"),
        ("fastify", "fastify"),
    ] {
        let declared = manifest["dependencies"][package]
            .as_str()
            .or_else(|| manifest["devDependencies"][package].as_str());
        let version = lock["packages"][format!("node_modules/{package}")]["version"]
            .as_str()
            .or_else(|| lock["dependencies"][package]["version"].as_str());
        if let Some(version) = version {
            output.push(pin(id, version, "package-lock.json"));
        } else if let Some(declared) = declared {
            output.push(requirement(
                id,
                declared,
                "package.json",
                Grammar::Constraint,
            ));
        }
    }
}

pub(super) fn detect(path: &Path) -> Result<Detected> {
    detect_with_origin(path, |_| None)
}

pub(super) fn detect_with_origin(
    path: &Path,
    origin_stack: impl Fn(&Path) -> Option<String>,
) -> Result<Detected> {
    let absolute = path
        .canonicalize()
        .with_context(|| format!("Open project path {}", path.display()))?;
    let start = if absolute.is_file() {
        absolute.parent().unwrap_or(&absolute)
    } else {
        &absolute
    };
    for current in start.ancestors().take(40) {
        let mut stacks: Vec<_> = MARKERS
            .iter()
            .filter(|(_, markers)| markers.iter().any(|name| current.join(name).is_file()))
            .map(|(stack, _)| *stack)
            .collect();
        let (projects, solution) = project_files(current);
        if (!projects.is_empty() || solution) && !stacks.contains(&"dotnet") {
            stacks.push("dotnet");
        }
        if stacks.is_empty() {
            if let Some(stack) = origin_stack(current).and_then(|origin| {
                MARKERS
                    .iter()
                    .find_map(|(stack, _)| (*stack == origin).then_some(*stack))
            }) {
                return Ok(Detected {
                    path: current.to_owned(),
                    stacks: vec![stack],
                    dependencies: vec![unversioned(stack, ".devkit-origin.json")],
                });
            }
            if current.join(".git").exists() {
                break;
            }
            continue;
        }
        stacks.sort_unstable();
        let mut dependencies = Vec::new();
        for stack in &stacks {
            match *stack {
                "node" => node(current, &mut dependencies),
                "go" => go(current, &mut dependencies),
                "rust" => rust(current, &mut dependencies),
                "swift" => swift(current, &mut dependencies),
                "python" => python(current, &mut dependencies),
                "ruby" => ruby(current, &mut dependencies),
                "java" => java(current, &mut dependencies),
                "elixir" => elixir(current, &mut dependencies),
                "dotnet" => dotnet(current, projects.clone(), solution, &mut dependencies),
                _ => unreachable!(),
            }
        }
        let mut unique = BTreeMap::new();
        for item in dependencies {
            unique.insert(item.id, item);
        }
        return Ok(Detected {
            path: current.to_owned(),
            stacks,
            dependencies: unique.into_values().collect(),
        });
    }
    Ok(Detected {
        path: start.to_owned(),
        stacks: Vec::new(),
        dependencies: Vec::new(),
    })
}
