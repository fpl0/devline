#!/usr/bin/env python3
"""Build immutable Tauri and Swift reference packs from official sources."""
from concurrent.futures import ThreadPoolExecutor
from hashlib import file_digest
from html import unescape
from pathlib import Path
from urllib.parse import unquote, urlsplit
from urllib.request import Request, urlopen
import argparse
import json
import re
import shutil
import tempfile
import textwrap
import tomllib

from reference_io import command, safe_extract_tar, write_shards
from reference_rendering import (
    MAX_IMAGE_BYTES, PANDOC_VERSION, DocumentParser, Node, as_text, clean_text, code_fence,
    front_matter, html_from_markdown, prepared_html, transform_prose, without_front_matter,
)

TAURI_COMMIT = "5e49cc6e48597060f1618e0b62cb91b071b1b4a9"
PLUGINS_COMMIT = "df594206ce03f67e25da04283dea023815a45058"
SWIFT_COMMIT = "0e8aa8c0fb7651714aaafa2c63a974d25244133e"
TAURI = "https://v2.tauri.app/"
SWIFT = "https://docs.swift.org/swift-book/"
DAY = "2026-11-07"
BASE = "/documentation/the-swift-programming-language"

INPUTS = {
    "tauri.tar.gz": (
        f"https://codeload.github.com/tauri-apps/tauri-docs/tar.gz/{TAURI_COMMIT}",
        "5cafc94ffe70fe856364f3d2e98d8ddece85330c5191f3fdbce228e09a0aef75",
    ),
    "tauri-plugins.tar.gz": (
        f"https://codeload.github.com/tauri-apps/plugins-workspace/tar.gz/{PLUGINS_COMMIT}",
        "3d75abf186e3b710f3691583a1eed8ca364b0dd11353e4ecf8c6d6045442437b",
    ),
    "swift.tar.gz": (
        f"https://codeload.github.com/swiftlang/swift-book/tar.gz/{SWIFT_COMMIT}",
        "eb99cd6223157b624e31cc1525c320e8fd1128e27a0a2ec39159c2af7ed81a4d",
    ),
    "tauri-config.json": (
        "https://schema.tauri.app/config/2.12.5",
        "ecac097c265293a8afdb6c01312d5db99299edfe7d8409f1bde02ce3fc27dafc",
    ),
    "tauri-core-LICENSE-MIT": (
        "https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.5/LICENSE-MIT",
        "9dd42ea92cff2ede5cd477cbfcce051b2d0115c0ac7f368ee88cb545055dff1d",
    ),
}


def record(key, title, source, content, aliases=(), category="guide"):
    text, html = content
    if not text.strip() or not html.strip():
        raise ValueError(f"Empty article {key}")
    return dict(id=key, title=title, body=text, html=html, source_url=source,
                aliases=list(dict.fromkeys([title, *aliases])), category=category)


def emit(pack_id, name, stack, upstream, source, license_name, overview, coverage,
         omissions, records, notices):
    version = upstream + "+devline.1"
    path = f"docs/{pack_id}/{version}"
    directory = output / path
    directory.mkdir(parents=True, exist_ok=False)
    if len({item["id"] for item in records}) != len(records):
        raise ValueError(f"Duplicate article IDs in {pack_id}")
    shards, size = write_shards(directory, records)
    for filename, path_source in notices.items():
        shutil.copyfile(path_source, directory / filename)
    (directory / "SOURCES.md").write_text(
        f"# {name}\n\nSnapshot {version}.\n\n{overview}\n\n"
        f"Source: {source}\n\nLicense: {license_name}.\n", encoding="utf-8",
    )
    print(f"{pack_id}: {len(records)} articles, {size:,} document bytes", flush=True)
    entry = dict(id=pack_id, name=name, stack=stack, version=version, description=coverage[0],
                 source_url=source, license=license_name, path=path, markdown=[],
                 upstream_version=upstream, snapshot_date=DAY, coverage=coverage, omissions=omissions)
    if shards > 1:
        entry["shards"] = shards
    return entry


def source_asset(base, raw):
    path = (base / unquote(urlsplit(raw).path).lstrip("/")).resolve()
    return path.read_bytes() if path.is_relative_to(base) and path.is_file() and path.stat().st_size <= MAX_IMAGE_BYTES else None


def swift_record(path):
    raw = json.loads(path.read_text())
    refs = raw.get("references", {})
    title = raw["metadata"]["title"]
    key = path.relative_to(root / "swift.doccarchive/data").as_posix().removeprefix("documentation/")
    route = "/" + key.removesuffix(".json")
    url = SWIFT + "documentation" + route
    assets = root / "swift.doccarchive"

    def url_for(ref):
        link = ref.get("url", "")
        return SWIFT + link.lstrip("/") if link.startswith("/documentation/") else link

    def nodes(items):
        return [render(item) for item in items]

    def render(item):
        kind = item["type"]
        if kind == "text":
            return item["text"]
        if kind == "codeVoice":
            return Node("code", children=[item["code"]])
        if kind in {"emphasis", "strong", "paragraph"}:
            return Node({"emphasis": "em", "strong": "strong", "paragraph": "p"}[kind],
                        children=nodes(item["inlineContent"]))
        if kind == "reference":
            ref = refs.get(item["identifier"], {})
            caption = item.get("overridingTitleInlineContent") or ref.get("titleInlineContent")
            label = nodes(caption) if caption else [item.get("overridingTitle") or ref.get("title") or item["identifier"]]
            return Node("a" if url_for(ref) else "span", {"href": url_for(ref)}, label)
        if kind == "heading":
            return Node(f"h{item['level']}", {"id": item.get("anchor", "")}, [item["text"]])
        if kind == "codeListing":
            language = item.get("syntax") or "swift"
            return Node("pre", children=[Node("code", {"class": f"language-{language}"}, ["\n".join(item["code"])])])
        if kind in {"unorderedList", "orderedList"}:
            tag = "ul" if kind == "unorderedList" else "ol"
            attributes = {"start": str(item["start"])} if "start" in item else {}
            return Node(tag, attributes, [Node("li", children=nodes(row["content"])) for row in item["items"]])
        if kind == "aside":
            return Node("blockquote", children=[Node("p", children=[Node("strong", children=[item["style"].title()])]),
                                                 *nodes(item["content"])])
        if kind == "image":
            ref = refs[item["identifier"]]
            variant = next((v for v in ref["variants"] if "light" in v["traits"]), ref["variants"][0])
            return Node("img", {"src": variant["url"], "alt": ref.get("alt") or item["identifier"]})
        if kind == "table":
            return Node("table", children=[Node("tr", children=[
                Node("th" if index == 0 and item["header"] == "row" else "td", children=nodes(cell))
                for cell in row]) for index, row in enumerate(item["rows"])])
        if kind == "termList":
            return Node("dl", children=[part for row in item["items"] for part in (
                Node("dt", children=nodes(row["term"]["inlineContent"])),
                Node("dd", children=nodes(row["definition"]["content"])))])
        raise ValueError(f"{key}: unsupported DocC content {kind}")

    body = [Node("h1", children=[title])]
    if raw.get("abstract"):
        body.append(Node("p", children=nodes(raw["abstract"])))
    for section in raw.get("primaryContentSections", []):
        if section["kind"] != "content":
            raise ValueError(f"{key}: unexpected DocC section {section['kind']}")
        body += nodes(section["content"])
    for section in raw.get("topicSections", []):
        body.append(Node("h2", children=[section["title"]]))
        body.append(Node("ul", children=[Node("li", children=[
            render({"type": "reference", "identifier": item})]) for item in section["identifiers"]]))
    if route == BASE:
        pages = sorted((root / "swift.doccarchive/data" / BASE.lstrip("/")).glob("*.json"))
        if not raw.get("topicSections"):
            body.append(Node("ul", children=[Node("li", children=[Node("a", {
                "href": SWIFT + BASE.lstrip("/") + "/" + page.stem},
                [json.loads(page.read_text())["metadata"]["title"]])]) for page in pages]))
    rendered = prepared_html(Node("", children=body), url, "swift",
                             assets=lambda raw: source_asset(assets, raw))
    categories = {"aboutswift", "compatibility", "revisionhistory", "lexicalstructure",
                  "types", "expressions", "statements", "declarations", "attributes",
                  "patterns", "genericparametersandarguments", "summaryofthegrammar"}
    aliases = {
        "thebasics": ("Optional", "if let", "guard let", "Int", "Bool"),
        "collectiontypes": ("Array", "Dictionary", "Set"),
        "concurrency": ("async await", "TaskGroup", "Sendable", "actor", "MainActor"),
        "enumerations": ("enum", "associated value", "CaseIterable"),
        "classesandstructures": ("struct", "class", "value type", "reference type"),
        "automaticreferencecounting": ("ARC", "weak", "unowned"),
        "opaquetypes": ("some", "any", "opaque result type"),
        "macros": ("@freestanding", "@attached"),
    }
    slug = path.stem
    category = "compatibility" if slug in {"compatibility", "revisionhistory"} else "reference" if slug in categories else "guide"
    return record(key, title, url, rendered, aliases.get(slug, ()), category)


def swift_pack():
    paths = sorted((root / "swift.doccarchive/data/documentation").rglob("*.json"))
    return emit("swift", "The Swift Programming Language", "swift", "6.4.0", SWIFT,
                "Apache-2.0 with Swift Runtime Library Exception",
                f"Official Swift 6.4 book from the [swift-6.4-fcs source tree](https://github.com/swiftlang/swift-book/tree/{SWIFT_COMMIT}). The 43 DocC articles retain their prose, examples, chapter links, and available inline illustrations.",
                ["Official Swift language tour, full language guide, grammar and language reference.",
                 "Optionals, collections, protocols, generics, concurrency, macros, and memory safety."],
                ["The Swift standard library API catalog and SwiftUI framework documentation are separate references."],
                [swift_record(path) for path in paths], {"LICENSE.txt": root / "swift/LICENSE.txt"})


def tauri_route(path):
    parts = [re.sub(r"[^a-z0-9]+", "-", part.lower()).strip("-") for part in path.with_suffix("").parts]
    if parts[-1] == "index":
        parts.pop()
    return "/".join(parts) + "/"


TAURI_ALIASES = {
    "start/create-project/": ("create-tauri-app", "cargo tauri dev", "npm create tauri-app"),
    "start/prerequisites/": ("macOS prerequisites", "Xcode Command Line Tools", "Rust", "Node.js"),
    "develop/calling-rust/": ("invoke", "tauri::command", "generate_handler!", "Command", "IPC"),
    "develop/calling-frontend/": ("Emitter", "Listener", "Channel", "emit", "emit_to", "listen", "unlisten"),
    "develop/state-management/": ("State", "Mutex", "manage", "AppHandle"),
    "develop/configuration-files/": ("tauri.conf.json", "tauri.macos.conf.json", "Tauri.toml"),
    "develop/tests/mocking/": ("mockIPC", "mockWindows", "mockConvertFileSrc"),
    "security/capabilities/": ("Capability", "desktop-schema.json", "ACL", "core:window:allow-set-title"),
    "security/permissions/": ("Permission", "PermissionSet", "allow commands", "deny commands"),
    "security/scope/": ("Scope", "allow scope", "deny scope"),
    "learn/window-customization/": ("Window", "WebviewWindow", "WindowBuilder", "decorations", "titleBarStyle"),
    "learn/system-tray/": ("TrayIcon", "TrayIconBuilder", "Menu"),
    "learn/window-menu/": ("Menu", "Submenu", "MenuItem"),
    "distribute/macos-application-bundle/": ("cargo tauri build", "macOS app bundle", "npm run tauri build"),
    "distribute/dmg/": ("DMG", "Apple disk image"),
}


def tauri_compatibility():
    workspace = root / "tauri-plugins"
    rust_version = tomllib.loads((workspace / "Cargo.toml").read_text())["workspace"]["package"]["rust-version"]
    tables = {}
    for path in sorted((workspace / "plugins").glob("*/Cargo.toml")):
        package = tomllib.loads(path.read_text())["package"]
        support = package.get("metadata", {}).get("platforms", {}).get("support")
        if support:
            specific = package.get("rust-version")
            tables[path.parent.name] = (specific if isinstance(specific, str) else rust_version, support)

    # The upstream compatibility generator adds these two built-in features.
    desktop_only = {platform: {"level": "full" if platform in {"windows", "linux", "macos"} else "none"}
                    for platform in ("windows", "linux", "macos", "android", "ios")}
    for feature in ("system-tray", "window-customization"):
        tables[feature] = rust_version, desktop_only
    return tables


def markdown_table(headers, rows):
    def row(cells):
        return "| " + " | ".join(cell.replace("|", r"\|").replace("\n", "<br>") for cell in cells) + " |"

    return "\n\n" + "\n".join([row(headers), row(["---"] * len(headers)), *(row(cells) for cells in rows)]) + "\n\n"


def support_label(info):
    level = info["level"]
    return {"full": "Full support", "partial": "Partial support", "none": "No support"}.get(level, f"Support: {level}")


def plugin_support(version, platforms):
    rows = [(platform, support_label(info), info.get("notes", "")) for platform, info in platforms.items()]
    return f"\n\n_This plugin requires a Rust version of at least **{version}**_" + markdown_table(
        ("Platform", "Level", "Notes"), rows)


def support_overview(tables):
    platforms = sorted({platform for _, support in tables.values() for platform in support})
    rows = []
    for plugin, (version, support) in sorted(tables.items()):
        cells = [plugin, version]
        for platform in platforms:
            info = support.get(platform)
            if info is None:
                cells.append("")
                continue
            label, notes = support_label(info), info.get("notes", "")
            cells.append(f"{label}: {notes}" if notes else label)
        rows.append(cells)
    return markdown_table(("Plugin", "Rust Version", *platforms), rows)


def unwrap_tauri_tabs(raw, path):
    def deindent(value):
        lines = value.splitlines(keepends=True)
        width = min((len(re.match(r"[ \t]*", line)[0].expandtabs(4)) for line in lines if line.strip()), default=0)
        if not width:
            return value

        def strip_indent(line):
            column = 0
            for index, char in enumerate(line):
                if char not in " \t":
                    break
                column += 1 if char == " " else 4 - column % 4
                if column >= width:
                    return " " * (column - width) + line[index + 1:]
            return line.lstrip(" \t")

        return "".join(strip_indent(line) for line in lines)

    frames = [("", "", "", [])]
    fence = None
    for line in raw.splitlines(keepends=True):
        marker = re.match(r"^[ \t]*(`{3,}|~{3,})([^\n]*)", line)
        if marker and (fence is None or marker[1].startswith(fence) and not marker[2].strip()):
            fence = marker[1] if fence is None else None
            frames[-1][3].append(line)
            continue
        tag = None if fence else re.fullmatch(r"([ \t]*)<(/?)(Tabs|TabItem)\b([^>]*)>\s*", line)
        if tag is None:
            frames[-1][3].append(line)
            continue
        indent, closing, name, attributes = tag.groups()
        if not closing:
            label = re.search(r"\blabel\s*=\s*([\"'])(.*?)\1", attributes)
            if name == "TabItem" and label is None:
                raise ValueError(f"{path}: unsupported TabItem label")
            frames.append((name, indent, label[2] if label else "", []))
            continue
        if frames[-1][0] != name:
            raise ValueError(f"{path}: unbalanced {name}")
        _, indent, label, lines = frames.pop()
        body = deindent("".join(lines)).strip("\n")
        if name == "TabItem":
            body = f"**{label}**\n\n{body}"
            if frames[-1][0] == "Tabs":
                indent = ""
        frames[-1][3].append("\n\n" + textwrap.indent(body, indent) + "\n\n")
    if len(frames) != 1:
        raise ValueError(f"{path}: unclosed {frames[-1][0]}")
    return "".join(frames[0][3])


def tauri_markdown(raw, path, known, compatibility):
    fragment = path.parent / "_sections/frontend-listen.mdx"
    plugin = front_matter(raw, "plugin")
    permissions = (root / "tauri-plugins/plugins" / plugin / "permissions/autogenerated/reference.md") if plugin else None
    def expand(prose):
        def commands(match):
            values = re.findall(r"\b(npm|yarn|pnpm|deno|bun|cargo)\s*=\s*(?:\{)?([\"'`])(.*?)\2", match[2], re.S)
            if not values:
                raise ValueError(f"{path}: could not expand CommandTabs")
            lines = "\n".join(f"**{name}**\n" + code_fence(unescape(code), "sh") for name, _, code in values)
            return "\n" + textwrap.indent(lines, match[1] or "") + "\n"

        def card(match):
            values = dict((name, value) for name, _, value in re.findall(
                r"\b(title|href)\s*=\s*([\"'])(.*?)\2", match[1], re.S))
            if values.keys() == {"title", "href"}:
                return f"\n- [{values['title']}]({values['href']})\n"
            return ""

        prose = re.sub(r"(?m)(^[ \t]*)?<CommandTabs\b(.*?)/>", commands, prose, flags=re.S)
        if "<FrontendListen" in prose:
            prose = re.sub(r"<FrontendListen\s*/>", lambda _: without_front_matter(fragment.read_text()), prose)
        prose = re.sub(r"<(?:LinkCard|LinkButton)\b(.*?)/>", card, prose, flags=re.S)
        prose = re.sub(r"<SinceVersion\b[^>]*?\bversion=([\"'])(.*?)\1[^>]*/>", r"\nAvailable since Tauri \2.\n", prose)
        prose = re.sub(r"^\s*import\s+.+?\bfrom\s+[\"'][^\"'\n]+[\"'];?[ \t]*$", "", prose, flags=re.S | re.M)
        prose = re.sub(r"\{\/\*.*?\*\/\}", "", prose, flags=re.S)
        if permissions and permissions.is_file():
            prose = re.sub(r"<PluginPermissions\b[^>]*/>", lambda _: permissions.read_text(), prose)
        if re.search(r"<Compatibility\b", prose):
            if plugin not in compatibility:
                raise ValueError(f"{path}: no upstream compatibility metadata for {plugin}")
            prose = re.sub(r"<Compatibility\b[^>]*/>", lambda _: plugin_support(*compatibility[plugin]), prose)
        if "<TableCompatibility" in prose:
            prose = re.sub(r"<TableCompatibility\s*/>", lambda _: support_overview(compatibility), prose)
        if path.name == "index.mdx" and path.parent.name == "plugin":
            pages = sorted(p for p in known if p.parent == path.parent and p != path)
            links = [f"- [{front_matter(p.read_text(), 'title') or p.stem}]({TAURI}plugin/{p.stem}/)" for p in pages]
            prose = prose.replace("<FeaturesList />", "\n" + "\n".join(links) + "\n")
            prose = prose.replace('<AwesomeTauri section="plugins-no-official" />',
                                  "[Community plugins](https://github.com/tauri-apps/awesome-tauri#plugins)")
            prose = prose.replace('<AwesomeTauri section="integrations" />',
                                  "[Community integrations](https://github.com/tauri-apps/awesome-tauri)")
            prose = prose.replace(r'Hover "\*" to see notes. For more details, visit the plugin page',
                                  "For more details, visit the plugin page")
        prose = re.sub(r"</?(?:Tabs|TabItem|Steps|ShowSolution|CardGrid|Card|Aside|Search)\b[^>]*>", "\n", prose)
        prose = re.sub(r"<(?:PluginLinks|PluginPermissions|Cta|CTA|AwesomeTauri|Icon|Image)\b[^>]*/>", "", prose)
        prose = re.sub(r"^[ \t]*:::(note|tip|caution|danger|warning|info)(?:\[([^\n\]]+)\])?[ \t]*$",
                       lambda x: f"\n**{x[2] or x[1].title()}**\n", prose, flags=re.M)
        prose = re.sub(r"^[ \t]*:::[ \t]*$", "", prose, flags=re.M)
        prose = re.sub(r"https://(?:tauri\.app|beta\.tauri\.app)/(?=(?:start|develop|learn|security|reference|distribute|concept|plugin)/)",
                       TAURI, prose)
        return prose

    value = transform_prose(unwrap_tauri_tabs(without_front_matter(raw), path), expand)
    return transform_prose(value, lambda prose: re.sub(
        r"(?m)^([ \t]*#{1,6})[ \t]+([^\n]+)$",
        lambda x: f"{x[1]} {x[2]} {{#{heading_id(x[2])}}}" if heading_id(x[2]) else x[0], prose))


def heading_id(text):
    text = re.sub(r"<[^>]+>|&[a-z]+;|\{#[^}]+\}", "", text)
    text = re.sub(r"[^\w\- ]", "", text.lower()).strip()
    return text.replace(" ", "-")


def tauri_html(prose, source, assets):
    body, markup = html_from_markdown(prose, source, "tauri", {}, assets)
    # MDX also accepts indented fences outside tabs. Pandoc can render those as
    # indented code containing the entire fence, even though the content is code.
    normalized = re.sub(
        r"<pre><code>(`{3,}|~{3,})(sh|bash|rust|json|toml|js|javascript|ts|typescript)(?:[ \t][^<\n]*)?\n([^<]*?)\n\1</code></pre>",
        r'<pre><code class="language-\2">\3</code></pre>', markup, flags=re.S,
    )
    if normalized == markup:
        return body, markup
    parser = DocumentParser()
    parser.feed(normalized)
    parser.close()
    return clean_text(as_text(parser.root)), normalized


def tauri_configuration(schema):
    def references(value):
        if isinstance(value, dict):
            reference = value.get("$ref", "")
            if reference.startswith("#/definitions/"):
                yield reference.rsplit("/", 1)[-1]
            for child in value.values():
                yield from references(child)
        elif isinstance(value, list):
            for child in value:
                yield from references(child)

    def details(value, omit):
        if isinstance(value, bool):
            return [code_fence(json.dumps(value), "json")]
        metadata = {key: item for key, item in value.items() if key not in omit}
        if not metadata:
            return []
        links = [f"[`{name}`](#{name.lower()})" for name in dict.fromkeys(references(metadata))]
        code = code_fence(json.dumps(metadata, ensure_ascii=False, indent=2), "json")
        return [", ".join(links), code] if links else [code]

    def section(name, value):
        yield f"## {name} {{#{name.lower()}}}"
        yield value.get("description", "")
        yield from details(value, {"description", "properties", "definitions"})
        for prop, settings in value.get("properties", {}).items():
            yield f"### {prop}"
            if isinstance(settings, dict):
                yield settings.get("description", "")
            yield from details(settings, {"description"})

    blocks = ["# Configuration", *section("Config", schema), "## Definitions {#definitions}"]
    for name, value in sorted(schema["definitions"].items()):
        blocks.extend(section(name, value))
    return "\n\n".join(block for block in blocks if block)


def tauri_pack():
    source = root / "tauri/src/content/docs"
    asset_root = root / "tauri/src/assets"
    compatibility = tauri_compatibility()
    directories = ("start", "develop", "learn", "security", "concept", "plugin")
    pages = {path for folder in directories for path in (source / folder).rglob("*.md*")
             if not any(p.startswith("_") for p in path.relative_to(source).parts)
             and "mobile" not in path.stem and "migrate" not in path.relative_to(source).parts}
    pages.update(source / file for file in (
        "distribute/index.mdx", "distribute/dmg.mdx", "distribute/macos-application-bundle.mdx",
        "distribute/app-store.mdx", "distribute/Sign/macos.mdx", "reference/environment-variables.mdx",
        "reference/webview-versions.md", "blog/tauri-2.12.mdx",
    ))

    def render(path):
        key = tauri_route(path.relative_to(source))
        raw = path.read_text()
        title = front_matter(raw, "title") or path.stem.replace("-", " ").title()
        url = TAURI + key
        prose = f"# {title}\n\n" + tauri_markdown(raw, path, pages, compatibility)
        def assets(raw):
            if raw.startswith("@assets/") and "logo" not in raw.lower():
                return source_asset(asset_root, raw.removeprefix("@assets/"))
            return None
        return record(key.removesuffix("/"), title, url, tauri_html(prose, url, assets),
                      TAURI_ALIASES.get(key, ()), "reference" if key.startswith("reference/") else "guide")

    with ThreadPoolExecutor(max_workers=8) as pool:
        records = list(pool.map(render, sorted(pages)))
    schema = root / "tauri-config.json"
    url = TAURI + "reference/config/"
    records.append(record("reference/config", "Configuration", url,
                          html_from_markdown(tauri_configuration(json.loads(schema.read_text())), url, "tauri", {}),
                          ("tauri.conf.json", "Config", "AppConfig", "BuildConfig", "BundleConfig", "WindowConfig",
                           "SecurityConfig", "Capability", "frontendDist", "devUrl", "beforeBuildCommand"), "reference"))
    return emit("tauri", "Tauri 2", "rust", "2.12.5", TAURI, "MIT; plugin reference files MIT or Apache-2.0",
                f"Official English Tauri 2 documentation, adapted from the [Tauri documentation source]("
                f"https://github.com/tauri-apps/tauri-docs/tree/{TAURI_COMMIT}). The configuration reference uses the "
                f"[Tauri 2.12.5 JSON Schema](https://schema.tauri.app/config/2.12.5), also preserved verbatim in `tauri-config-2.12.5.json`. "
                "Descriptions remain prose and schema constraints and defaults appear in JSON blocks. Plugin permissions and compatibility "
                "tables come from the "
                f"[upstream plugin workspace](https://github.com/tauri-apps/plugins-workspace/tree/{PLUGINS_COMMIT}) "
                "pinned by that documentation checkout. Compatibility notes are visible in the tables. Package manager tabs are "
                f"expanded into separate examples. Snapshot date {DAY}.",
                ["Official Tauri 2 guides and versioned application configuration reference.",
                 "Project setup, Rust commands, JavaScript events and channels, capabilities and permissions, native windows, testing, and macOS distribution.",
                 "Official plugin platform support, compatibility notes, and minimum Rust versions."],
                ["Interactive site controls, two MDX illustrations, localized pages, migration guides, mobile-specific guides, and generated plugin API catalogs are outside this snapshot."],
                records, {"LICENSE": root / "tauri/LICENSE", "SCHEMA-LICENSE-MIT": root / "tauri-core-LICENSE-MIT",
                          "tauri-config-2.12.5.json": schema,
                          "PLUGIN-LICENSE-MIT": root / "tauri-plugins/LICENSE_MIT",
                          "PLUGIN-LICENSE-APACHE": root / "tauri-plugins/LICENSE_APACHE-2.0"})


def fetch(cache, name, *, online):
    url, expected = INPUTS[name]
    path = cache / name

    def matches(file):
        with file.open("rb") as content:
            return file_digest(content, "sha256").hexdigest() == expected

    if path.exists():
        if not matches(path):
            raise RuntimeError(f"Unexpected SHA-256 for cached source {path}")
        return path
    if not online:
        raise RuntimeError(f"Missing source {path}. Run again with --fetch while connected.")
    request = Request(url, headers={"User-Agent": "Devline Desktop Documentation Builder"})
    with tempfile.NamedTemporaryFile(dir=cache, delete=False) as file:
        temporary = Path(file.name)
        try:
            with urlopen(request, timeout=90) as response:
                shutil.copyfileobj(response, file)
            file.flush()
            if not matches(temporary):
                raise RuntimeError(f"Unexpected SHA-256 from {url}")
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)
    return path


def prepare_inputs(cache, *, online):
    with ThreadPoolExecutor(max_workers=4) as pool:
        files = dict(zip(INPUTS, pool.map(lambda name: fetch(cache, name, online=online), INPUTS)))
    for name, prefix, dest in (
        ("tauri.tar.gz", f"tauri-docs-{TAURI_COMMIT}", "tauri"),
        ("tauri-plugins.tar.gz", f"plugins-workspace-{PLUGINS_COMMIT}", "tauri-plugins"),
        ("swift.tar.gz", f"swift-book-{SWIFT_COMMIT}", "swift"),
    ):
        safe_extract_tar(files[name], root, lambda path: path.parts[0] == prefix)
        (root / prefix).rename(root / dest)
    for name in ("tauri-config.json", "tauri-core-LICENSE-MIT"):
        shutil.copyfile(files[name], root / name)
    command("xcrun", "docc", "convert", str(root / "swift/TSPL.docc"),
            "--output-path", str(root / "swift.doccarchive"),
            "--fallback-display-name", "The Swift Programming Language",
            "--fallback-bundle-identifier", "org.swift.tspl",
            "--fallback-bundle-version", "6.4")


def main():
    global root, output
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog=f"Use Xcode 27.2, Pandoc {PANDOC_VERSION}, and a fresh output directory. Add --fetch to populate the cache.",
    )
    parser.add_argument("--cache", required=True, type=Path, help="cache for checksum-pinned upstream sources")
    parser.add_argument("--output", required=True, type=Path, help="write both packs and docs-catalog.json here")
    parser.add_argument("--fetch", action="store_true", help="download missing pinned source inputs over HTTPS")
    opts = parser.parse_args()
    output = opts.output.resolve()
    for name in ("docs-catalog.json", "docs/tauri", "docs/swift"):
        if (output / name).exists():
            parser.error("Choose a fresh output directory to preserve immutable snapshots.")
    if command("pandoc", "--version").splitlines()[0] != f"pandoc {PANDOC_VERSION}":
        parser.error(f"Pandoc {PANDOC_VERSION} is required to reproduce these snapshots.")
    if command("xcodebuild", "-version").splitlines() != ["Xcode 27.2", "Build version 27B52"]:
        parser.error("Xcode 27.2 (27B52) is required to reproduce the Swift DocC snapshot.")
    cache = opts.cache.resolve()
    cache.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="devline-desktop-docs-") as scratch:
        root = Path(scratch).resolve()
        prepare_inputs(cache, online=opts.fetch)
        entries = [tauri_pack(), swift_pack()]
        (output / "docs-catalog.json").write_text(json.dumps(entries, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
