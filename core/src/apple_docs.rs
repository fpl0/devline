use crate::{
    doc_library::install_doc_pack, doc_links::SDK_LINK_PREFIX, doc_updates, lock_library, markdown,
    tasks::Job, templates, Context, DocPack,
};
use anyhow::{bail, Context as _, Result};
use pulldown_cmark::{Event, Parser, Tag};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::{BufReader, BufWriter, Write as _},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub(crate) const PACK_ID: &str = "swiftui";
const NAME: &str = "SwiftUI SDK Reference";
const REFERENCE: &str = "https://developer.apple.com/documentation/swiftui";
const FORMAT: u8 = 2;
const MAX_GRAPH_BYTES: u64 = 64 * 1024 * 1024;

struct Sdk {
    path: PathBuf,
    swift: PathBuf,
    sdk_version: String,
    build: String,
    toolchain: String,
    version: String,
    modules: Vec<&'static str>,
}

fn output(command: &mut Command) -> Result<String> {
    let result = command.stdin(Stdio::null()).output()?;
    if !result.status.success() {
        let detail = String::from_utf8_lossy(&result.stderr);
        bail!("{}", detail.trim().chars().take(280).collect::<String>());
    }
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

fn xcrun(sdk: &str, arguments: &[&str]) -> Result<String> {
    output(
        Command::new("/usr/bin/xcrun")
            .args(["--sdk", sdk])
            .args(arguments),
    )
    .context("Select an installed macOS SDK with xcode-select or DEVELOPER_DIR")
}

fn framework(sdk: &Path, module: &str) -> bool {
    sdk.join("System/Library/Frameworks")
        .join(format!("{module}.framework/Modules/{module}.swiftmodule"))
        .canonicalize()
        .is_ok_and(|path| path.starts_with(sdk) && path.is_dir())
}

fn detected_sdk() -> Result<Sdk> {
    let path = PathBuf::from(xcrun("macosx", &["--show-sdk-path"])?);
    let selected = path.to_str().context("SDK path is not UTF-8")?;
    let swift = PathBuf::from(xcrun(selected, &["--find", "swift"])?);
    let sdk_version = xcrun(selected, &["--show-sdk-version"])?;
    let build = xcrun(selected, &["--show-sdk-build-version"])?;
    if !path.is_absolute() || !swift.is_absolute() || swift.file_name() != Some("swift".as_ref()) {
        bail!("xcrun did not identify a macOS SDK and Swift driver");
    }
    let path = path.canonicalize()?;
    let executable = fs::metadata(&swift)?;
    if path.extension().and_then(|value| value.to_str()) != Some("sdk")
        || !framework(&path, "SwiftUI")
        || !executable.is_file()
        || executable.permissions().mode() & 0o111 == 0
    {
        bail!("The selected macOS SDK does not contain a usable SwiftUI module and Swift driver");
    }
    if sdk_version.len() > 32
        || sdk_version.is_empty()
        || !sdk_version
            .split('.')
            .all(|part| part.parse::<u32>().is_ok())
        || build.is_empty()
        || build.len() > 32
        || !build.bytes().all(|byte| byte.is_ascii_alphanumeric())
    {
        bail!("xcrun returned an invalid macOS SDK identity");
    }
    let toolchain = output(Command::new(&swift).arg("--version"))?
        .lines()
        .find(|line| line.contains("Swift version "))
        .filter(|line| line.len() <= 512)
        .context("Swift driver did not report its compiler version")?
        .to_owned();
    let modules = if framework(&path, "SwiftUICore") {
        vec!["SwiftUI", "SwiftUICore"]
    } else {
        vec!["SwiftUI"]
    };
    let fingerprint = format!(
        "{FORMAT}\n{toolchain}\n{}\n{modules:?}",
        std::env::consts::ARCH
    );
    let digest = format!("{:x}", Sha256::digest(fingerprint));
    let version = format!(
        "{sdk_version}+sdk.{}.{}",
        build.to_ascii_lowercase(),
        &digest[..12]
    );
    Ok(Sdk {
        path,
        swift,
        sdk_version,
        build,
        toolchain,
        version,
        modules,
    })
}

fn installed_count(db: &rusqlite::Connection, version: &str) -> Result<Option<u64>> {
    let pack = doc_updates::installed_metadata(db)?;
    if !pack
        .get(&(PACK_ID.to_owned(), version.to_owned()))
        .is_some_and(|metadata| metadata.local_only)
    {
        return Ok(None);
    }
    Ok(db
        .query_row(
            "SELECT count FROM packs WHERE id=?1 AND version=?2",
            params![PACK_ID, version],
            |row| row.get(0),
        )
        .optional()?)
}

pub(crate) fn status(ctx: &Context) -> Result<Value> {
    let db = ctx.database()?;
    let active = doc_updates::preferences(&db)?.remove(PACK_ID);
    match detected_sdk() {
        Ok(sdk) => {
            let count = installed_count(&db, &sdk.version)?;
            Ok(
                json!({"id": PACK_ID, "name": NAME, "local_only": true, "available": true,
                "version": sdk.version, "sdk_version": sdk.sdk_version, "sdk_build": sdk.build,
                "toolchain_version": sdk.toolchain, "installed": count.is_some(), "count": count,
                "active_version": active, "detail": "Apple declarations and documentation comments from this Mac’s selected SDK."}),
            )
        }
        Err(error) => {
            let count = active
                .as_deref()
                .map(|version| installed_count(&db, version))
                .transpose()?
                .flatten();
            Ok(
                json!({"id": PACK_ID, "name": NAME, "local_only": true, "available": false,
                "installed": count.is_some(), "count": count, "active_version": active, "detail": format!("{error:#}")}),
            )
        }
    }
}

#[derive(Deserialize)]
struct Graph {
    module: GraphModule,
    symbols: Vec<Symbol>,
}
#[derive(Deserialize)]
struct GraphModule {
    name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Symbol {
    identifier: Identifier,
    names: Names,
    path_components: Vec<String>,
    #[serde(default)]
    declaration_fragments: Vec<Fragment>,
    doc_comment: Option<Comment>,
}
#[derive(Deserialize)]
struct Identifier {
    precise: String,
}
#[derive(Deserialize)]
struct Names {
    title: String,
}
#[derive(Deserialize)]
struct Fragment {
    spelling: String,
}
#[derive(Deserialize)]
struct Comment {
    lines: Vec<Line>,
}
#[derive(Deserialize)]
struct Line {
    text: String,
}

struct Source {
    path: Vec<String>,
    title: String,
    module: &'static str,
    declaration: String,
    comment: String,
}

fn source(symbol: Symbol, module: &'static str) -> Option<(String, Source)> {
    let declaration: String = symbol
        .declaration_fragments
        .into_iter()
        .map(|fragment| fragment.spelling)
        .collect();
    let comment = symbol.doc_comment.map_or_else(String::new, |comment| {
        comment
            .lines
            .into_iter()
            .map(|line| line.text)
            .collect::<Vec<_>>()
            .join("\n")
    });
    if symbol.identifier.precise.is_empty()
        || symbol.names.title.is_empty()
        || symbol.path_components.is_empty()
        || (declaration.trim().is_empty() && comment.trim().is_empty())
    {
        return None;
    }
    Some((
        symbol.identifier.precise,
        Source {
            path: symbol.path_components,
            title: symbol.names.title,
            module,
            declaration,
            comment,
        },
    ))
}

fn extract(
    sdk: &Sdk,
    stage: &Path,
    module: &'static str,
    job: Option<&mut Job>,
) -> Result<BTreeMap<String, Source>> {
    let output = stage.join(module);
    fs::create_dir(&output)?;
    let stage_path =
        serde_json::to_string(stage.to_str().context("SDK staging path is not UTF-8")?)?;
    let policy = format!("(version 1)(allow default)(deny file-write*)(allow file-write* (subpath {stage_path}) (literal \"/dev/null\"))(deny network-outbound)");
    let architecture = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let target = format!("{architecture}-apple-macos{}", sdk.sdk_version);
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", stage.join("home"))
        .env("TMPDIR", stage.join("tmp"))
        .args(["-p", &policy])
        .arg(&sdk.swift)
        .args([
            "symbolgraph-extract",
            "-module-name",
            module,
            "-target",
            &target,
            "-sdk",
        ])
        .arg(&sdk.path)
        .arg("-module-cache-path")
        .arg(stage.join("module-cache"))
        .args([
            "-minimum-access-level",
            "public",
            "-skip-inherited-docs",
            "-skip-synthesized-members",
            "-output-dir",
        ])
        .arg(&output);
    templates::run_command(
        &mut command,
        &format!("Extracting Apple {module} SDK symbols"),
        job,
    )?;
    let mut paths = fs::read_dir(&output)?
        .map(|entry| entry.map(|item| item.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort_unstable();
    if paths.len() > 64 || !paths.contains(&output.join(format!("{module}.symbols.json"))) {
        bail!("Swift symbol graph extraction did not produce the expected {module} files");
    }
    let mut sources = BTreeMap::new();
    for path in paths {
        let meta = fs::symlink_metadata(&path)?;
        if !meta.is_file()
            || meta.len() > MAX_GRAPH_BYTES
            || !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".symbols.json"))
        {
            bail!("Swift symbol graph extraction returned an invalid file");
        }
        let graph: Graph = serde_json::from_reader(BufReader::new(fs::File::open(&path)?))?;
        if graph.module.name != module {
            bail!("The Swift symbol graph module did not match {module}");
        }
        merge(
            &mut sources,
            graph
                .symbols
                .into_iter()
                .filter_map(|symbol| source(symbol, module)),
        );
    }
    Ok(sources)
}

fn merge(sources: &mut BTreeMap<String, Source>, more: impl IntoIterator<Item = (String, Source)>) {
    for (id, candidate) in more {
        use std::collections::btree_map::Entry;
        match sources.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
            Entry::Occupied(mut entry)
                if entry.get().comment.trim().is_empty()
                    && !candidate.comment.trim().is_empty() =>
            {
                entry.insert(candidate);
            }
            Entry::Occupied(_) => {}
        }
    }
}

fn explicit_apple_reference(path: &str) -> String {
    let mut url = "https://developer.apple.com/documentation".to_owned();
    for part in path.split('/') {
        url.push('/');
        for byte in part.to_lowercase().bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~():,".contains(&byte) {
                url.push(char::from(byte));
            } else {
                write!(url, "%{byte:02X}").expect("writing to a string cannot fail");
            }
        }
    }
    url
}

fn routes(sources: &BTreeMap<String, Source>) -> BTreeMap<String, Option<String>> {
    let mut result = BTreeMap::new();
    for (identifier, item) in sources {
        let hash = symbol_hash(identifier);
        let url = format!("{SDK_LINK_PREFIX}{hash}");
        let keys = BTreeSet::from([item.path.join("/"), item.title.clone()]);
        for key in keys {
            result
                .entry(key)
                .and_modify(|existing| *existing = None)
                .or_insert_with(|| Some(url.clone()));
        }
    }
    result
}

fn show_code_tabs(comment: &str) -> String {
    let mut visible = String::new();
    let mut remaining = comment;
    while let Some((before, block)) = remaining.split_once("\n@TabNavigator {\n") {
        let Some((tabs, after)) = block.split_once("\n}\n") else {
            break;
        };
        let Some((code, preview)) = tabs
            .strip_prefix("    @Tab(\"Code\") {\n")
            .and_then(|tabs| tabs.split_once("\n    }\n"))
        else {
            break;
        };
        if !code.starts_with("        ```")
            || !code.ends_with("\n        ```")
            || !preview.starts_with("    @Tab(\"Preview\") {\n")
        {
            break;
        }
        visible.push_str(before);
        visible.push_str("\n\n");
        for line in code.lines() {
            visible.push_str(line.strip_prefix("        ").unwrap_or(line));
            visible.push('\n');
        }
        visible.push('\n');
        remaining = after;
    }
    visible.push_str(remaining);
    visible
}

fn link_comment(comment: &str, routes: &BTreeMap<String, Option<String>>) -> String {
    let mut rendered = String::new();
    let mut end = 0;
    for (event, range) in Parser::new(comment).into_offset_iter() {
        if range.start < end {
            continue;
        }
        let raw = &comment[range.clone()];
        let label = match &event {
            Event::Code(code) if raw.starts_with("``") => code.as_ref(),
            Event::Start(Tag::Link { .. }) | Event::InlineHtml(_)
                if raw.starts_with("<doc:") && raw.ends_with('>') =>
            {
                &raw[5..raw.len() - 1]
            }
            _ => continue,
        };
        let apple_path = label.strip_prefix("//com.apple.documentation/documentation/");
        let key = apple_path.unwrap_or(label);
        let key = key
            .strip_prefix("SwiftUI/")
            .or_else(|| key.strip_prefix("SwiftUICore/"))
            .unwrap_or(key);
        let target = routes
            .get(key)
            .and_then(Option::as_ref)
            .cloned()
            .or_else(|| apple_path.map(explicit_apple_reference));
        if label.contains(['`', '[', ']', '\n']) || (target.is_none() && !raw.starts_with("<doc:"))
        {
            continue;
        }
        rendered.push_str(&comment[end..range.start]);
        if let Some(target) = target {
            write!(rendered, "[`{label}`](<{target}>)").expect("writing to a string cannot fail");
        } else {
            write!(rendered, "`{label}`").expect("writing to a string cannot fail");
        }
        end = range.end;
    }
    rendered.push_str(&comment[end..]);
    rendered
}

#[derive(Serialize)]
struct Record {
    id: String,
    title: String,
    body: String,
    html: String,
    source_url: String,
    aliases: Vec<String>,
    category: &'static str,
}

fn symbol_hash(identifier: &str) -> String {
    format!("{:x}", Sha256::digest(identifier))
}

fn record(
    sdk: &Sdk,
    identifier: &str,
    symbol: &Source,
    routes: &BTreeMap<String, Option<String>>,
    top_level: &BTreeSet<String>,
) -> Record {
    let hash = symbol_hash(identifier);
    let title = symbol.path.join(".");
    let notice = if symbol.comment.trim().is_empty() {
        "This SDK provides a declaration only for this symbol.\n\n"
    } else {
        ""
    };
    let provenance = format!("\n\n{notice}## SDK provenance\n\nApple macOS SDK {} ({}), module `{}`.\n\nSDK symbol: `{identifier}`.",
        sdk.sdk_version, sdk.build, symbol.module);
    let declaration = if symbol.declaration.is_empty() {
        String::new()
    } else {
        format!("```swift\n{}\n```\n\n", symbol.declaration)
    };
    let body = format!("{declaration}{}{provenance}", symbol.comment);
    let visible = format!(
        "{declaration}{}{provenance}",
        link_comment(&show_code_tabs(&symbol.comment), routes)
    );
    let mut aliases = vec![format!("SwiftUI.{title}")];
    if symbol.title != title && !top_level.contains(&symbol.title.to_lowercase()) {
        aliases.push(symbol.title.clone());
    }
    let (html, _) = markdown::render(&visible, REFERENCE, None);
    Record {
        id: format!("symbol/{hash}"),
        title,
        body,
        html,
        source_url: REFERENCE.into(),
        aliases,
        category: "api",
    }
}

fn pack(sdk: &Sdk, total: usize, documented: usize) -> DocPack {
    DocPack {
        id: PACK_ID.into(), name: NAME.into(), stack: "swift".into(), version: sdk.version.clone(),
        path: format!("docs/{PACK_ID}/{}", sdk.version), source_url: REFERENCE.into(),
        description: format!("Apple SwiftUI reference extracted from the selected macOS {} SDK ({}).", sdk.sdk_version, sdk.build),
        license: "Apple SDK documentation. Local use under the applicable Apple SDK terms.".into(),
        markdown: Some(vec!["symbol/*".into()]), shards: None,
        upstream_version: Some(sdk.sdk_version.clone()), snapshot_date: None,
        coverage: vec![format!("{total} public declarations, including {documented} with original Apple SDK documentation comments.")],
        omissions: vec!["Apple Developer website articles, tutorials, and illustrations are outside this SDK snapshot. Some SDK symbols have declarations only.".into()],
        local_only: true,
    }
}

pub(crate) fn install(
    ctx: &Context,
    requested: Option<&str>,
    mut job: Option<&mut Job>,
) -> Result<Value> {
    if let Some(version) = requested {
        let _lock = lock_library(ctx)?;
        if let Some((pack, path)) = doc_updates::cached_pack(ctx, PACK_ID, Some(version))? {
            if pack.local_only {
                return install_doc_pack(ctx, &pack, &path);
            }
        }
    }
    if let Some(job) = job.as_deref_mut() {
        job.stage("Locating the selected Apple macOS SDK")?;
    }
    let sdk = detected_sdk()?;
    if requested.is_some_and(|version| version != sdk.version) {
        bail!(
            "That SwiftUI SDK snapshot is unavailable. The selected SDK produces {}",
            sdk.version
        );
    }
    {
        let _lock = lock_library(ctx)?;
        if let Some((pack, path)) = doc_updates::cached_pack(ctx, PACK_ID, Some(&sdk.version))? {
            if pack.local_only {
                return install_doc_pack(ctx, &pack, &path);
            }
        }
    }
    let parent = templates::library(ctx, Path::new("apple-sdk-import"), true)?;
    let stage = templates::Stage::new(&parent)?;
    fs::set_permissions(&stage.0, fs::Permissions::from_mode(0o700))?;
    for name in ["home", "tmp", "module-cache", "pack"] {
        fs::create_dir(stage.0.join(name))?;
    }
    let mut sources = BTreeMap::new();
    for module in &sdk.modules {
        merge(
            &mut sources,
            extract(&sdk, &stage.0, module, job.as_deref_mut())?,
        );
    }
    let documented = sources
        .values()
        .filter(|source| !source.comment.trim().is_empty())
        .count();
    if documented == 0 {
        bail!("The selected SDK did not expose any SwiftUI documentation comments");
    }
    if let Some(job) = job.as_deref_mut() {
        job.stage("Rendering Apple SDK declarations and comments")?;
    }
    let routes = routes(&sources);
    let top_level = sources
        .values()
        .filter(|symbol| symbol.path.len() == 1)
        .map(|symbol| symbol.title.to_lowercase())
        .collect();
    let mut records = Vec::with_capacity(sources.len());
    for (index, (identifier, symbol)) in sources.iter().enumerate() {
        if index % 256 == 0 {
            if let Some(job) = job.as_deref_mut() {
                job.checkpoint()?;
            }
        }
        records.push(record(&sdk, identifier, symbol, &routes, &top_level));
    }
    let package = stage.0.join("pack");
    let mut file = BufWriter::new(fs::File::create(package.join("documents.json"))?);
    serde_json::to_writer(&mut file, &records)?;
    file.flush()?;
    let pack = pack(&sdk, records.len(), documented);
    fs::write(package.join("SOURCES.md"), format!(
        "# {NAME}\n\nApple macOS SDK {} ({}).\n\nCompiler: {}.\n\nModules: {}.\n\nPublic declarations and original documentation comments extracted with Swift symbolgraph-extract. Each article identifies its SDK module and precise symbol. The Source link opens Apple’s contextual SwiftUI reference.\n\nReference: {REFERENCE}\n\n{}\n",
        sdk.sdk_version, sdk.build, sdk.toolchain, sdk.modules.join(", "), pack.license,
    ))?;
    if let Some(job) = job {
        job.stage("Indexing the local SwiftUI SDK reference")?;
    }
    let _lock = lock_library(ctx)?;
    let cached = doc_updates::store_original(ctx, &pack, &package)?;
    let mut result = install_doc_pack(ctx, &pack, &cached)?;
    result["local_only"] = json!(true);
    result["documented_count"] = json!(documented);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol(path: &[&str], comment: &str) -> Source {
        Source {
            path: path.iter().map(|value| (*value).to_owned()).collect(),
            title: path.last().unwrap().to_string(),
            module: "SwiftUICore",
            declaration: "public struct Fixture".into(),
            comment: comment.into(),
        }
    }

    fn fixture_sdk() -> Sdk {
        Sdk {
            path: PathBuf::new(),
            swift: PathBuf::new(),
            sdk_version: "27.2".into(),
            build: "26B118".into(),
            toolchain: "Fixture compiler".into(),
            version: "27.2+sdk.fixture".into(),
            modules: vec!["SwiftUICore"],
        }
    }

    #[test]
    fn formats_sdk_links_without_changing_original_comments_or_code() {
        let sdk = fixture_sdk();
        let originals = BTreeMap::from([
            ("view-body".into(), symbol(&["View", "body"], "")),
            ("view-body-type".into(), symbol(&["View", "Body"], "")),
            ("ambiguous-a".into(), symbol(&["Text", "init(_:)"], "")),
            ("ambiguous-b".into(), symbol(&["Text", "init(_:)"], "")),
        ]);
        let source = "See ``View/body``, ``Text/init(_:)``, and <doc://com.apple.documentation/documentation/Foundation/Date>.\n\n    let literal = \"``View/body``\"\n\n@TabNavigator {\n    @Tab(\"Code\") {\n        ```swift\n        ViewExample()\n        ```\n    }\n    @Tab(\"Preview\") {\n        @Image(source: preview)\n    }\n}\n\nAfter tabs.";
        let record = record(
            &sdk,
            "fixture",
            &symbol(&["Fixture"], source),
            &routes(&originals),
            &BTreeSet::new(),
        );
        assert!(record.body.contains(source));
        assert!(record.body.contains("module `SwiftUICore`"));
        assert!(record
            .html
            .contains(&format!("#devline-sdk-symbol-{}", symbol_hash("view-body"))));
        assert!(!record.html.contains(&symbol_hash("ambiguous-a")));
        assert!(record
            .html
            .contains("https://developer.apple.com/documentation/foundation/date"));
        assert!(record.html.contains("``View/body``"));
        assert!(record.html.contains("ViewExample()"));
        assert!(!record.html.contains("@TabNavigator"));
        assert_eq!(record.source_url, REFERENCE);
    }

    #[test]
    fn deduplicates_precise_ids_and_keeps_documented_symbol() {
        let mut sources = BTreeMap::new();
        merge(&mut sources, [("same".into(), symbol(&["View"], ""))]);
        merge(
            &mut sources,
            [("same".into(), symbol(&["View"], "Original Apple comment"))],
        );
        merge(
            &mut sources,
            [("same".into(), symbol(&["View"], "Other graph comment"))],
        );
        assert_eq!(sources.len(), 1);
        assert_eq!(sources["same"].comment, "Original Apple comment");
    }

    #[test]
    fn top_level_types_outrank_member_aliases_and_normalized_callables() -> Result<()> {
        let sdk = fixture_sdk();
        let stage = templates::Stage::new(&std::env::temp_dir())?;
        let home = stage.0.join("library");
        fs::create_dir(&home)?;
        let context = Context {
            home,
            catalog_dir: stage.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        let symbols = BTreeMap::from([
            ("type", symbol(&["State"], "Documented type")),
            (
                "member",
                symbol(&["AnimationContext", "state"], "Documented member"),
            ),
            ("callable", symbol(&["State()"], "Documented callable")),
            (
                "noncolliding",
                symbol(&["View", "padding"], "Documented modifier"),
            ),
        ]);
        let names = BTreeSet::from(["state".into(), "state()".into()]);
        let records: Vec<_> = symbols
            .iter()
            .map(|(id, source)| record(&sdk, id, source, &BTreeMap::new(), &names))
            .collect();
        fs::write(
            stage.0.join("documents.json"),
            serde_json::to_vec(&records)?,
        )?;
        install_doc_pack(
            &context,
            &pack(&sdk, records.len(), records.len()),
            &stage.0,
        )?;
        for (query, expected) in [
            ("State", "State"),
            ("SwiftUI.State", "State"),
            ("SwiftUI.AnimationContext.state", "AnimationContext.state"),
            ("SwiftUI.State()", "State()"),
            ("padding", "View.padding"),
        ] {
            let doc = crate::doc_search::show(&context, &json!({"id": query, "pack_id": PACK_ID}))?;
            assert_eq!(doc["title"], expected);
        }
        Ok(())
    }
}
