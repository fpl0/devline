#!/usr/bin/env python3
"""Build Devline offline docs from pinned upstream artifacts and local toolchains.

See --help for source inputs. This script never changes the input archives,
reference tree, or toolchain installations. Pandoc 3.13 renders Markdown at build
time. The resulting snapshots have no runtime rendering dependency.
"""

import argparse
import ast
import base64
import copy
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from filecmp import cmp
from html.parser import HTMLParser
import html
import json
import os
from pathlib import Path
import posixpath
import re
import shutil
import subprocess
import tarfile
import tempfile
import textwrap
from urllib.parse import parse_qs, quote, unquote, urljoin, urlsplit
from urllib.request import Request, urlopen
import zipfile
import xml.etree.ElementTree as ET


DOTNET_COMMIT = "37fe0bfc5f596bf0bafd74bf927010384f184d96"
ASPNET_COMMIT = "47d6f5c46f7a4c552fbf8ca9077439b95150c719"
ASPNET_SAMPLES_COMMIT = "8dbb6f86c84f78cfd4a61f9207d55199635595bf"
ASPNET_SAMPLES_ARCHIVE = f"aspnet-samples-{ASPNET_SAMPLES_COMMIT}.tar.gz"

DOWNLOADS = {
    "python.zip": "https://docs.python.org/3.15/archives/python-3.15-docs-html.zip",
    "python-LICENSE": "https://raw.githubusercontent.com/python/cpython/v3.15.2/LICENSE",
    "fastapi-0.162.0.tar.gz": "https://codeload.github.com/fastapi/fastapi/tar.gz/refs/tags/0.162.0",
    "ruby.tar.gz": "https://cache.ruby-lang.org/pub/ruby/4.0/ruby-4.0.9.tar.gz",
    "rails.tar.gz": "https://codeload.github.com/rails/rails/tar.gz/refs/tags/v8.1.3",
    "elixir-docs.tar.gz": "https://repo.hex.pm/docs/elixir-1.20.5.tar.gz",
    "elixir-LICENSE": "https://raw.githubusercontent.com/elixir-lang/elixir/v1.20.5/LICENSE",
    "phoenix-LICENSE": "https://raw.githubusercontent.com/phoenixframework/phoenix/v1.8.16/LICENSE.md",
    "ecto-LICENSE": "https://raw.githubusercontent.com/elixir-ecto/ecto/v3.14.3/LICENSE.md",
    "spring-LICENSE": "https://raw.githubusercontent.com/spring-projects/spring-boot/v4.1.2/LICENSE.txt",
    "dotnet-docs-full.tar.gz": f"https://codeload.github.com/dotnet/docs/tar.gz/{DOTNET_COMMIT}",
    "aspnet-docs-full.tar.gz": f"https://codeload.github.com/dotnet/AspNetCore.Docs/tar.gz/{ASPNET_COMMIT}",
    ASPNET_SAMPLES_ARCHIVE: f"https://codeload.github.com/dotnet/AspNetCore.Docs.Samples/tar.gz/{ASPNET_SAMPLES_COMMIT}",
}

VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}
OMIT = {"audio", "button", "canvas", "embed", "footer", "form", "head", "iframe", "input",
        "nav", "noscript", "object", "script", "style", "svg", "template", "video"}
BLOCK = {"article", "blockquote", "dd", "div", "dl", "dt", "h1", "h2", "h3", "h4", "h5", "h6",
         "hr", "li", "main", "ol", "p", "pre", "section", "summary", "table", "tr", "ul"}
PRESENTATION = {"headerlink", "visually-hidden", "sr-only", "sidebar", "copy-button", "copy-path",
                "breadcrumb", "breadcrumbs", "pagination", "screen-reader-only", "toc", "source-link",
                "search-toggle", "view-source", "icon-action", "skip-link", "hover-link"}
SAFE_TAGS = {"a", "abbr", "article", "aside", "b", "blockquote", "br", "caption", "code", "col",
             "colgroup", "dd", "del", "details", "dfn", "div", "dl", "dt", "em", "figcaption",
             "figure", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "i", "kbd", "li", "mark", "ol",
             "p", "pre", "s", "samp", "section", "small", "span", "strong", "sub", "summary", "sup",
             "table", "tbody", "td", "th", "thead", "tfoot", "tr", "u", "ul", "var", "wbr"}
SAFE_CLASSES = {"admonition", "admonition-title", "attention", "caution", "danger", "error",
                "example", "hint", "important", "info", "note", "tip", "warning", "versionadded",
                "versionchanged", "deprecated", "versionmodified", "sourceCode", "sig", "signature"}
CONTENT_REVISION = "devline.1"
CONTENT_REVISIONS = {
    "csharp": "devline.4", "aspnet": "devline.3", "python": "devline.2",
    "go": "devline.2", "rails": "devline.2", "ruby": "devline.2",
}
PANDOC_VERSION = "3.13"
MAX_IMAGE_BYTES = 750_000
SHARD_BYTES = 64 * 1024 * 1024
IMAGE_SUFFIXES = {".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp"}


@dataclass
class Node:
    tag: str
    attrs: dict = field(default_factory=dict)
    children: list = field(default_factory=list)

    def walk(self):
        yield self
        for item in self.children:
            if isinstance(item, Node):
                yield from item.walk()


class DocumentParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.root = Node("")
        self.stack = [self.root]

    def handle_starttag(self, tag, attrs):
        node = Node(tag, dict(attrs))
        self.stack[-1].children.append(node)
        if tag not in VOID:
            self.stack.append(node)

    def handle_startendtag(self, tag, attrs):
        self.stack[-1].children.append(Node(tag, dict(attrs)))

    def handle_endtag(self, tag):
        for index in range(len(self.stack) - 1, 0, -1):
            if self.stack[index].tag == tag:
                del self.stack[index:]
                break

    def handle_data(self, data):
        self.stack[-1].children.append(data)


def as_text(node, pre=False):
    if isinstance(node, str):
        return node if pre else re.sub(r"\s+", " ", node)
    if node.tag in OMIT or PRESENTATION.intersection((node.attrs.get("class") or "").split()):
        return ""
    if node.tag == "br":
        return "\n"
    if node.tag == "img":
        return node.attrs.get("alt", "")
    text = "".join(as_text(child, pre or node.tag == "pre") for child in node.children)
    if node.tag == "pre":
        return "\n\n" + textwrap.indent(text.rstrip("\n"), "    ", lambda _: True) + "\n\n"
    if node.tag == "li":
        return "\n• " + text.strip() + "\n"
    if node.tag in {"td", "th"}:
        return text.strip() + "  "
    if node.tag in BLOCK:
        return "\n\n" + text + "\n\n"
    return text


def clean_text(text):
    text = re.sub(r"[ \t]+\n", "\n", text)
    return re.sub(r"\n{3,}", "\n\n", text).strip()


def link_destination(raw, source, pack_id, xrefs):
    if not raw or raw.startswith("//") or re.search(r"[\x00-\x20\\]", raw):
        return None
    if raw.startswith("#"):
        return raw
    try:
        target = urlsplit(raw)
        if target.scheme == "xref":
            uid = unquote(target.path)
            if not re.fullmatch(r"[\w.+`<>{},()\[\]-]+", uid):
                return None
            return xrefs.get(uid) or "https://learn.microsoft.com/en-us/dotnet/api/" + quote(uid.lower(), safe=".`-")
        if not target.scheme and target.path.endswith((".md", ".mdx")):
            path = target.path.rsplit(".", 1)[0]
            if pack_id == "fastapi":
                source = source.rstrip("/")
                path = path.removesuffix("/index") + "/" if path != "index" else "./"
            elif pack_id == "rails":
                path += ".html"
            raw = target._replace(path=path).geturl()
        if raw.startswith("~/") and pack_id in {"csharp", "aspnet"}:
            route = "dotnet" if pack_id == "csharp" else "aspnet/core"
            raw = f"https://learn.microsoft.com/en-us/{route}/" + raw[2:]
        absolute = urljoin(source, raw)
        target = urlsplit(absolute)
        if target.scheme not in {"http", "https"} or not target.hostname or target.username is not None:
            return None
        return absolute
    except ValueError:
        return None


def image_url(content):
    if not content or len(content) > MAX_IMAGE_BYTES:
        return None
    mime = next((kind for prefix, kind in (
        (b"\x89PNG\r\n\x1a\n", "png"), (b"\xff\xd8\xff", "jpeg"),
        (b"GIF87a", "gif"), (b"GIF89a", "gif"),
    ) if content.startswith(prefix)), None)
    if content[:4] == b"RIFF" and content[8:12] == b"WEBP":
        mime = "webp"
    if not mime and safe_svg(content):
        mime = "svg+xml"
    return f"data:image/{mime};base64," + base64.b64encode(content).decode("ascii") if mime else None


def safe_svg(content):
    if re.search(br"<!\s*(?:doctype|entity)|<\?xml-stylesheet", content, re.I):
        return False
    try:
        root = ET.fromstring(content)
    except (ET.ParseError, UnicodeError):
        return False
    if root.tag.rsplit("}", 1)[-1] != "svg":
        return False
    for node in root.iter():
        name = node.tag.rsplit("}", 1)[-1].lower()
        if name in {"script", "foreignobject", "image", "iframe", "object", "embed", "a", "animate",
                    "set", "animatetransform", "animatemotion"}:
            return False
        for key, value in node.attrib.items():
            key = key.rsplit("}", 1)[-1].lower()
            if key.startswith("on") or key in {"src", "base"} or key == "href" and not value.startswith("#"):
                return False
        styles = [*node.attrib.values(), node.text or "" if name == "style" else ""]
        for style in styles:
            if "@" in style or "\\" in style or any(not value.strip(" \"'").startswith("#")
                                   for value in re.findall(r"url\s*\(([^)]*)\)", style, re.I)):
                return False
    return True


def asset_path(raw):
    try:
        uri = urlsplit(raw)
    except ValueError:
        return None
    path = unquote(uri.path)
    if uri.scheme or uri.netloc or "\\" in path or "\x00" in path:
        return None
    return path if Path(path).suffix.lower() in IMAGE_SUFFIXES else None


def file_assets(file, root, alternate=None):
    root = root.resolve()

    def load(raw):
        path = asset_path(raw)
        if not path:
            return None
        candidates = [root / path.removeprefix("~/").lstrip("/")] if path.startswith(("/", "~/")) else [file.parent / path]
        if alternate:
            candidates.append(alternate / path.lstrip("/"))
        for candidate in candidates:
            candidate = candidate.resolve()
            if (candidate.is_relative_to(root) and candidate.is_file()
                    and candidate.stat().st_size <= MAX_IMAGE_BYTES):
                return candidate.read_bytes()
        return None

    return load


def archive_assets(archive, prefix, key):
    def load(raw):
        path = asset_path(raw)
        if not path:
            return None
        relative = posixpath.normpath(posixpath.join(posixpath.dirname(key), path)) if not path.startswith("/") else path[1:]
        if relative.startswith(("../", "/")):
            return None
        try:
            member = archive.getinfo(prefix + "/" + relative)
        except KeyError:
            return None
        return archive.read(member) if member.file_size <= MAX_IMAGE_BYTES else None

    return load


def sanitize(node, source, pack_id, xrefs, assets=None):
    if isinstance(node, str):
        return node
    classes = (node.attrs.get("class") or "").split()
    if node.tag in OMIT or PRESENTATION.intersection(classes):
        return None
    if node.tag == "a" and ("toc-backref" in classes or
                            pack_id in {"csharp", "aspnet"} and (node.attrs.get("href") or "").startswith("#tab/")):
        return Node("", children=[value for item in node.children
                                  if (value := sanitize(item, source, pack_id, xrefs, assets)) is not None])
    if node.tag == "img":
        raw = node.attrs.get("src") or ""
        alt = node.attrs.get("alt") or ""
        embedded = image_url(assets(raw)) if assets else None
        if embedded:
            return Node("img", {"src": embedded, "alt": alt})
        href = link_destination(raw, source, pack_id, xrefs)
        label = alt or "Upstream image"
        return Node("a", {"href": href}, [label]) if href else Node("span", children=[label])
    attrs = {}
    anchor = unquote(node.attrs.get("id") or (node.attrs.get("name") if node.tag == "a" else "") or "")
    if anchor and len(anchor) <= 500 and not re.search(r"[\x00-\x20]", anchor):
        attrs["id"] = anchor
    classes = [value for value in classes if value in SAFE_CLASSES or (
        node.tag == "code" and re.fullmatch(r"(?:language-)?[a-zA-Z][a-zA-Z0-9+#.-]{0,24}", value))]
    if classes:
        attrs["class"] = " ".join(classes)
    if node.tag == "a":
        raw = node.attrs.get("href") or ""
        href = link_destination(raw, source, pack_id, xrefs)
        if href:
            attrs["href"] = href
    for key in {"colspan", "rowspan"} if node.tag in {"td", "th"} else {"start"} if node.tag == "ol" else ():
        value = node.attrs.get(key)
        if value and re.fullmatch(r"[0-9]{1,4}", value):
            attrs[key] = value
    if node.tag == "details" and "open" in node.attrs:
        attrs["open"] = ""
    children = [value for item in node.children if (value := sanitize(item, source, pack_id, xrefs, assets)) is not None]
    tag = node.tag if node.tag in SAFE_TAGS else "span" if anchor else ""
    if tag == "span" and not attrs:
        tag = ""
    return Node(tag, attrs, children)


def render_html(node):
    if isinstance(node, str):
        return html.escape(node, quote=False)
    inner = "".join(map(render_html, node.children))
    if not node.tag:
        return inner
    attrs = "".join(f' {key}="{html.escape(value, quote=True)}"' for key, value in sorted(node.attrs.items()))
    return f"<{node.tag}{attrs}>" + ("" if node.tag in VOID else f"{inner}</{node.tag}>")


def prepared_html(node, source, pack_id, xrefs=None, assets=None):
    normalized = sanitize(node, source, pack_id, xrefs or {}, assets)
    if pack_id == "ruby":
        ruby_label_links(normalized, source)
    return clean_text(as_text(normalized)), render_html(normalized).strip()


def ruby_label_links(article, source):
    ids = {node.attrs.get("id") for node in article.walk()}
    for node in article.walk():
        href = node.attrs.get("href", "")
        if not href.startswith("#"):
            continue
        fragment = unquote(href[1:])
        if fragment in ids:
            continue
        if fragment.endswith("]") and fragment[:-1] in ids:
            node.attrs["href"] = "#" + fragment[:-1]
            continue
        label = re.fullmatch(r"(?:module-(\w+)-)?label-(.+)", fragment)
        if not label:
            continue
        owner, words = label.groups()
        slug = words.lower().replace("+", "-")
        if slug in ids:
            node.attrs["href"] = "#" + slug
        elif owner == "JSON" and urlsplit(source).path.endswith("/JSON.html"):
            # ri prints several JSON methods separately from the class sections they cite.
            node.attrs["href"] = source.partition("#")[0] + "#" + slug


def rails_headings(root):
    """Match the Rails guide renderer's canonical slugs and parent-qualified duplicates."""
    used = {}
    anchors = {}
    hierarchy = []

    def with_parent(slug, path):
        return f"{anchors[id(path[-2])]}-{slug}" if len(path) > 1 else slug

    for heading in root.children:
        if not isinstance(heading, Node) or heading.tag not in {"h2", "h3", "h4", "h5"}:
            continue
        text = as_text(heading).lower().replace("?", "-questionmark").replace("!", "-bang").replace("[]", "-squarebrackets")
        slug = re.sub(r"[\\/`*_{}\[\]()#+\-.!:,;|&<>^~='\"=]+", " ", text).strip()
        slug = re.sub(r"\s+", "-", slug)
        if not slug:
            continue
        hierarchy = hierarchy[:int(heading.tag[1]) - 2] + [heading]
        previous = used.get(slug)
        if previous:
            if len(previous) > 1:
                prior_slug = with_parent(slug, previous)
                anchors[id(previous[-1])] = prior_slug
                used[prior_slug] = used.pop(slug)
            slug = with_parent(slug, hierarchy)
        used[slug] = hierarchy
        anchors[id(heading)] = slug

    ids = {node.attrs.get("id") for node in root.walk()}
    for heading in root.children:
        canonical = anchors.get(id(heading))
        if not canonical or canonical in ids:
            continue
        if old := heading.attrs.get("id"):
            heading.children.insert(0, Node("span", {"id": old}))
        heading.attrs["id"] = canonical
        ids.add(canonical)


def go_grammar_anchors(article):
    ids = {node.attrs.get("id") for node in article.walk()}
    for node in article.walk():
        if node.tag != "pre" or "ebnf" not in (node.attrs.get("class") or "").split():
            continue
        parts = []
        for item in node.children:
            if not isinstance(item, str):
                parts.append(item)
                continue
            for index, part in enumerate(re.split(r"(?m)^([A-Za-z]\w*)(?=[ \t]*=)", item)):
                if index % 2 and part not in ids:
                    parts.append(Node("span", {"id": part}, [part]))
                    ids.add(part)
                else:
                    parts.append(part)
        node.children = parts


def article_from_html(raw, mode, title_fallback):
    parser = DocumentParser()
    parser.feed(raw)
    nodes = list(parser.root.walk())
    heading = next((n for n in nodes if n.tag == "title"), None)
    title = clean_text("".join(str(t) for t in heading.children)) if heading else title_fallback
    title = re.sub(r"\s+(?:[—–]\s+(?:Python .+ documentation|[\w .-]+ v\d[\w.]*)(?:\s+—\s+Documentation)?|"
                   r":: Spring Boot|- Rust|\(Java SE [^)]+\))$", "", title).strip()
    if mode == "sphinx":
        main = next((n for n in nodes if n.attrs.get("role") == "main"), None)
    elif mode == "hex":
        main = next((n for n in nodes if n.attrs.get("id") == "content"), None)
    elif mode == "spring":
        main = next((n for n in nodes if n.tag == "article" and "doc" in (n.attrs.get("class") or "").split()), None)
    else:
        main = next((n for n in nodes if n.tag == "main"), None)
    if main is None:
        main = next((n for n in nodes if n.tag == "body"), parser.root)
    aliases = []
    for n in main.walk():
        key = unquote(n.attrs.get("id") or "")
        if ((n.tag == "dt" and "." in key)
                or (n.tag in {"h2", "h3", "section", "a"} and re.fullmatch(
                    r"(?:(?:method|tymethod|associatedconstant|fn)\.[\w]+|[^\s/]+/\d+)", key))
                or (mode == "javadoc" and n.tag == "section" and "(" in key)):
            aliases.append(key)
    return title or title_fallback, main, aliases


def read_text(file):
    return file.read_text(encoding="utf-8-sig", errors="replace")


def markdown_article(raw, fallback):
    raw = without_front_matter(raw)
    raw = re.sub(r"\A\*\*DO NOT READ THIS FILE ON GITHUB.*?\*\*\s*", "", raw, flags=re.S)
    title = re.search(r"^#\s+([^\n]+)|^([^\n]+)\n=+[ \t]*$", raw, re.M)
    title = (title.group(1) or title.group(2)).strip() if title else fallback
    title = re.sub(r"\s+\{\s*#[\w.-]+\s*\}\s*$", "", title)
    return title, raw


def without_front_matter(raw):
    return re.sub(r"\A\s*---\s*\n.*?\n---[ \t]*\n", "", raw, count=1, flags=re.S)


def code_fence(content, language):
    fence = "`" * max(3, 1 + max((len(run) for run in re.findall(r"`+", content)), default=0))
    return f"\n{fence}{language}\n{content.rstrip()}\n{fence}\n"


def html_from_markdown(raw, source, pack_id, xrefs, assets=None):
    result = subprocess.run(
        ("pandoc", "--from=gfm+attributes+fenced_divs", "--to=html5", "--syntax-highlighting=none"),
        input=raw, text=True, capture_output=True, check=False,
    )
    if result.returncode:
        raise RuntimeError(f"Pandoc could not render {source}: {result.stderr.strip()}")
    parser = DocumentParser()
    parser.feed(result.stdout)
    parser.close()
    if pack_id == "rails":
        rails_headings(parser.root)
    return prepared_html(parser.root, source, pack_id, xrefs, assets)


def slug_title(file):
    return file.stem.replace("-", " ").replace("_", " ")


def document_category(pack, key):
    area, _, rest = key.partition("/")
    if pack == "csharp" and key.startswith(("core/compatibility/", "core/breaking-changes/")):
        return "compatibility"
    if pack == "python":
        if area == "library":
            return "api"
        if area in {"tutorial", "howto", "using", "installing", "distributing"}:
            return "guide"
    if pack == "fastapi":
        return "api" if area == "reference" else "guide"
    if pack == "go" and area == "std" or pack == "chi" and area == "api":
        return "api"
    if pack == "chi":
        return "guide"
    if pack in {"java", "axum"}:
        return "api"
    if pack == "spring-boot" and area in {"how-to", "tutorial"}:
        return "guide"
    if pack == "ruby":
        return "api" if area in {"api", "method"} else "guide"
    if pack == "rails":
        return "guide"
    if pack == "rust":
        if area == "std":
            return "api"
        if area in {"book", "edition-guide", "rust-by-example"} or key.startswith("cargo/guide/"):
            return "guide"
    if pack in {"elixir", "phoenix", "ecto"}:
        return "api" if key[:1].isupper() else "guide"
    if pack == "csharp" and rest.startswith(("programming-guide/", "tutorials/", "whats-new/tutorial")):
        return "guide"
    if pack == "aspnet" and area in {"host-and-deploy", "test", "tutorials"}:
        return "guide"
    return "reference"


@dataclass
class Pack:
    id: str
    name: str
    stack: str
    version: str
    description: str
    source_url: str
    license: str
    source_detail: str
    documents: list = field(default_factory=list)
    notices: dict = field(default_factory=dict)
    markdown: list = field(default_factory=list)
    xrefs: dict = field(default_factory=dict)
    missing: set = field(default_factory=set)
    attachments: dict = field(default_factory=dict)

    def add(self, key, title, body, source, aliases=(), markup=None):
        if len(body) < 40:
            return
        self.documents.append({
            "id": key,
            "title": title,
            "body": body,
            "html": markup or "<pre>" + html.escape(body) + "</pre>",
            "source_url": source,
            "aliases": list(dict.fromkeys(a for a in aliases if a and len(a) < 200)),
            "category": document_category(self.id, key),
        })

    def add_markdown(self, key, title, raw, source, aliases=(), assets=None):
        self.markdown.append((key, title, raw, source, aliases, assets))

    def render_markdown(self, page):
        key, title, raw, source, aliases, assets = page
        body, markup = html_from_markdown(raw, source, self.id, self.xrefs, assets)
        return key, title, body, source, aliases, markup

    def notice(self, path, name=None):
        self.notices[name or path.name] = read_text(path)

    def emit(self, data):
        if self.markdown:
            pandoc = shutil.which("pandoc")
            if not pandoc or command(pandoc, "--version").splitlines()[0] != f"pandoc {PANDOC_VERSION}":
                raise RuntimeError(f"Pandoc {PANDOC_VERSION} is required to reproduce the Markdown snapshots")
        with ThreadPoolExecutor(max_workers=min(8, os.cpu_count() or 1)) as pool:
            for page in pool.map(self.render_markdown, self.markdown):
                self.add(*page)
        if not self.documents:
            raise RuntimeError(f"{self.id} has no documents")
        ids = [document["id"] for document in self.documents]
        if len(ids) != len(set(ids)):
            raise RuntimeError(f"Duplicate document IDs in {self.id}")
        snapshot_version = f"{self.version}+{CONTENT_REVISIONS.get(self.id, CONTENT_REVISION)}"
        target = data / "docs" / self.id / snapshot_version
        target.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix=".snapshot-", dir=target.parent) as scratch:
            staging = Path(scratch) / "snapshot"
            staging.mkdir()
            shard_count, document_bytes = write_shards(staging, self.documents)
            for name, content in self.notices.items():
                (staging / name).write_text(content, encoding="utf-8")
            for name, path in self.attachments.items():
                shutil.copyfile(path, staging / name)
            (staging / "SOURCES.md").write_text(
                f"# {self.name} {self.version}\n\nDevline snapshot: {snapshot_version}\n\n"
                f"Source: {self.source_url}\n\n"
                f"{self.source_detail}\n\nLicense: {self.license}. See the accompanying notices. "
                "Devline retained article text, code, headings, links and other structural markup. "
                "Local illustrations are embedded where available. Interactive controls and site chrome are omitted. Markdown was rendered "
                f"with Pandoc {PANDOC_VERSION}. Each article links to its upstream source.\n",
                encoding="utf-8",
            )
            if self.missing:
                (staging / "MISSING-INCLUDES.txt").write_text(
                    "Upstream files or snippets unavailable in the supplied source snapshot:\n\n"
                    + "\n".join(sorted(self.missing)) + "\n", encoding="utf-8",
                )
            if target.exists():
                names = {path.name for path in staging.iterdir()}
                if names != {path.name for path in target.iterdir()} or any(
                    not cmp(staging / name, target / name, shallow=False) for name in names
                ):
                    raise RuntimeError(f"Immutable snapshot changed: {target}. Use a new content revision.")
            else:
                staging.rename(target)
        if self.missing:
            print(f"{self.id:<13} {len(self.missing):>5} unavailable includes/snippets (see MISSING-INCLUDES.txt)", flush=True)
        print(f"{self.id:<13} {len(self.documents):>5} records {document_bytes / 2**20:>6.1f} MiB", flush=True)
        metadata = {
            "id": self.id, "name": self.name, "stack": self.stack,
            "version": snapshot_version, "description": self.description,
            "source_url": self.source_url, "license": self.license,
            "path": target.relative_to(data).as_posix(),
        }
        metadata["markdown"] = []
        if shard_count > 1:
            metadata["shards"] = shard_count
        return metadata


def atomic_json(path, value):
    temporary = path.with_suffix(".tmp")
    with temporary.open("w", encoding="utf-8") as output:
        output.write("[\n")
        for index, item in enumerate(value):
            if index:
                output.write(",\n")
            json.dump(item, output, ensure_ascii=False, separators=(",", ":"))
        output.write("\n]\n")
    temporary.replace(path)


def write_shards(target, documents):
    """Bound source archive files while keeping individual articles intact."""
    number = 1
    total = 0
    shard = []
    size = 4
    for document in documents:
        encoded = json.dumps(document, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        if len(encoded) + 4 > SHARD_BYTES:
            raise RuntimeError(f"Article exceeds shard size: {document['id']}")
        if shard and size + len(encoded) + 2 > SHARD_BYTES:
            total += write_shard(target, number, shard)
            number += 1
            shard = []
            size = 4
        shard.append(encoded)
        size += len(encoded) + 2
    total += write_shard(target, number, shard)
    return number, total


def write_shard(target, number, rows):
    path = target / ("documents.json" if number == 1 else f"documents-{number}.json")
    temporary = path.with_suffix(".tmp")
    with temporary.open("wb") as output:
        output.write(b"[\n")
        for index, row in enumerate(rows):
            if index:
                output.write(b",\n")
            output.write(row)
        output.write(b"\n]\n")
    temporary.replace(path)
    return path.stat().st_size


def safe_extract_tar(path, destination, select):
    with tarfile.open(path) as archive:
        for entry in archive:
            name = Path(entry.name)
            if not entry.isfile() or name.is_absolute() or ".." in name.parts or not select(name):
                continue
            target = destination / name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(entry) as source, target.open("wb") as output:
                shutil.copyfileobj(source, output)


def command(*args, cwd=None, env=None, check=True):
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True, check=False)
    if check and result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-1600:]}")
    return result.stdout if result.returncode == 0 else ""


def html_page(pack, raw, key, *, mode="main", source=None, title_prefix="", aliases=(), assets=None):
    title, main, extracted = article_from_html(raw, mode, slug_title(Path(key)))
    if pack.id == "go" and key == "doc/go_spec.html":
        go_grammar_anchors(main)
    source = source or pack.source_url + quote(key, safe="/.-")
    body, markup = prepared_html(main, source, pack.id, assets=assets)
    if body:
        qualified = []
        if aliases and mode == "hex":
            qualified = [f"{aliases[0]}.{name}" for item in extracted for name in (item, item.partition("/")[0])]
        elif aliases and mode == "javadoc":
            qualified = [f"{name}.{item.partition('(')[0]}" for name in aliases for item in extracted]
        elif aliases and (pack.stack == "rust" or pack.id == "axum"):
            qualified = [f"{name}::{item.partition('.')[2]}" for name in aliases for item in extracted if "." in item]
        pack.add(key, title_prefix + title, body, source, [*aliases, *qualified, *extracted], markup)


def python_pack(cache):
    pack = Pack("python", "Python", "python", "3.15.2",
                "Official language reference, tutorial and standard library manual.",
                "https://docs.python.org/3.15/", "PSF-2.0; examples also 0BSD",
                "Python Software Foundation, Python 3.15.2 HTML documentation archive. "
                "https://docs.python.org/3.15/download.html")
    pack.notice(cache / "python-LICENSE", "LICENSE")
    groups = {"library", "reference", "tutorial", "using", "howto", "installing", "distributing"}
    with zipfile.ZipFile(cache / "python.zip") as archive:
        index = next(name for name in archive.namelist() if name.count("/") == 1 and name.endswith("/index.html"))
        if "Python 3.15.2 documentation" not in archive.read(index).decode():
            raise RuntimeError("Python HTML archive no longer corresponds to 3.15.2")
        for name in sorted(archive.namelist()):
            key = name.partition("/")[2]
            file = Path(key)
            if file.suffix != ".html" or file.parts[0] not in groups:
                continue
            if file.stem in {"index", "genindex"}:
                continue
            raw = archive.read(name).decode()
            html_page(pack, raw, key, mode="sphinx", aliases=(file.stem,),
                      assets=archive_assets(archive, name.partition("/")[0], key))
    return pack


def include_fastapi_examples(raw, root, key, missing):
    def substitute(match):
        name = match.group(1).strip().split()[0]
        base = (root / "docs_src").resolve()
        example = (base / name).resolve()
        if not example.is_relative_to(base) or not example.is_file():
            missing.add(f"{key}: docs_src/{name}")
            return "\n> Example source was unavailable in this snapshot.\n"
        return code_fence(read_text(example), example.suffix.removeprefix(".").replace("py", "python"))

    return re.sub(r"\{\*\s+(?:\.\./)+docs_src/(.*?)\s+\*\}", substitute, raw)


def material_blocks(raw):
    """Convert Material for MkDocs blocks, leaving verbatim fences alone."""
    output = []
    blocks = []
    code = None
    for line in raw.splitlines():
        fence = re.match(r"^\s*(`{3,}|~{3,})", line)
        if fence:
            marker = fence[1]
            if code is None:
                code = marker
            elif marker.startswith(code) and not line.strip().strip(code[0]):
                code = None
            output.append(line)
            continue
        directive = None if code else re.fullmatch(r"\s*(/{3,})(?:\s+([\w-]+)(?:\s*\|\s*(.*))?)?\s*", line)
        if directive:
            marker, kind, label = directive.groups()
            if kind:
                delimiter = ":" * (len(marker) + 1)
                blocks.append((marker, delimiter))
                kind = kind.lower()
                classes = f".admonition .{kind}" if kind != "tab" else ".tab"
                output.extend(("", delimiter + " {" + classes + "}", "",
                               f"**{label or kind.replace('-', ' ').title()}**", ""))
                continue
            if blocks and blocks[-1][0] == marker:
                output.extend(("", blocks.pop()[1], ""))
                continue
        output.append(line)
    for _, delimiter in reversed(blocks):
        output.extend(("", delimiter))
    return "\n".join(output) + "\n"


def fastapi_symbols(root, starlette):
    modules = {}
    for file in sorted((root / "fastapi").rglob("*.py")):
        parts = file.relative_to(root).with_suffix("").parts
        name = ".".join(parts[:-1] if parts[-1] == "__init__" else parts)
        modules[name] = ast.parse(read_text(file))
    with zipfile.ZipFile(starlette) as archive:
        for file in sorted(archive.namelist()):
            if file.startswith("starlette/") and file.endswith(".py"):
                parts = Path(file).with_suffix("").parts
                name = ".".join(parts[:-1] if parts[-1] == "__init__" else parts)
                modules[name] = ast.parse(archive.read(file))
    symbols = dict(modules)
    imports = {}
    for module, tree in modules.items():
        for node in tree.body:
            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
                symbols[f"{module}.{node.name}"] = node
            elif isinstance(node, (ast.Assign, ast.AnnAssign)):
                for name in node.targets if isinstance(node, ast.Assign) else [node.target]:
                    if isinstance(name, ast.Name) and not name.id.startswith("_"):
                        symbols[f"{module}.{name.id}"] = node
            elif isinstance(node, ast.ImportFrom) and node.module:
                if node.level == 0:
                    origin = node.module
                elif node.level == 1:
                    origin = (module if (root / module.replace(".", "/")).is_dir()
                              else module.rpartition(".")[0]) + "." + node.module
                else:
                    continue
                for name in node.names:
                    imports[f"{module}.{name.asname or name.name}"] = f"{origin}.{name.name}"
    for alias, target in imports.items():
        visited = {alias}
        while target in imports and target not in symbols and target not in visited:
            visited.add(target)
            target = imports[target]
        if target in symbols:
            symbols.setdefault(alias, symbols[target])
    return symbols


def api_description(name, node):
    description = ""
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
        args = copy.deepcopy(node.args)
        for argument in [*args.posonlyargs, *args.args, *args.kwonlyargs, args.vararg, args.kwarg]:
            if argument and isinstance(argument.annotation, ast.Subscript) and (
                isinstance(argument.annotation.value, ast.Name) and argument.annotation.value.id == "Annotated"
                and isinstance(argument.annotation.slice, ast.Tuple)
            ):
                argument.annotation = argument.annotation.slice.elts[0]
        async_prefix = "async " if isinstance(node, ast.AsyncFunctionDef) else ""
        returns = " -> " + ast.unparse(node.returns) if node.returns else ""
        signature = f"{async_prefix}def {node.name}({ast.unparse(args)}){returns}"
        description = ast.get_docstring(node) or ""
    elif isinstance(node, ast.ClassDef):
        base = ", ".join(map(ast.unparse, node.bases))
        signature = f"class {node.name}" + (f"({base})" if base else "")
        description = ast.get_docstring(node) or ""
    elif isinstance(node, ast.Module):
        signature = f"import {name}"
        description = ast.get_docstring(node) or ""
        exports = [f"`{item.name}`" for item in node.body
                   if isinstance(item, (ast.ClassDef, ast.FunctionDef)) and not item.name.startswith("_")]
        if exports:
            description += "\n\n**Public declarations:** " + ", ".join(exports)
    else:
        signature = ast.unparse(node)
    return (
        f"\n### `{name}` {{#{name}}}\n" + code_fence(signature, "python")
        + "\n" + description + "\n"
    )


def fastapi_pack(cache, scratch, starlette):
    pack = Pack("fastapi", "FastAPI", "python", "0.162.0",
                "Official FastAPI tutorial, advanced user guide and deployment guides.",
                "https://fastapi.tiangolo.com/", "MIT; Starlette BSD-3-Clause",
                "FastAPI contributors, repository tag 0.162.0. "
                "https://github.com/fastapi/fastapi/tree/0.162.0/docs/en/docs\n\n"
                "Re-exported Starlette 1.10.0 reference signatures and docstrings come from the pinned "
                "starter wheel. https://www.starlette.io/")
    safe_extract_tar(cache / "fastapi-0.162.0.tar.gz", scratch, lambda name:
                     name.name == "LICENSE" or str(name).partition("/")[2].startswith(("docs/en/docs/", "docs_src/", "fastapi/")))
    root = scratch / "fastapi-0.162.0"
    pack.notice(root / "LICENSE")
    with zipfile.ZipFile(starlette) as archive:
        license_file = next(name for name in archive.namelist() if name.endswith("/licenses/LICENSE.md"))
        pack.notices["STARLETTE-LICENSE.md"] = archive.read(license_file).decode()
    base = root / "docs/en/docs"
    symbols = fastapi_symbols(root, starlette)
    for file in sorted(base.rglob("*.md")):
        key = file.relative_to(base).as_posix()
        if key.startswith(("release-notes", "external-links", "_", "img/", "benchmarks", "translation-banner")):
            continue
        raw = include_fastapi_examples(read_text(file), root, key, pack.missing)
        aliases = []
        if key.startswith("reference/"):
            def inline_api(match):
                name = match[1]
                if name not in symbols:
                    pack.missing.add(f"{key}: API source for {name}")
                    return f"\n> API source for `{name}` is available in the upstream reference.\n"
                aliases.extend((name, name.rpartition(".")[2]))
                return api_description(name, symbols[name])

            raw = re.sub(r"^:::\s+([\w.]+)[ \t]*\n(?:^[ \t]+[^\n]*\n|^\n(?=^[ \t]+))*",
                         inline_api, raw, flags=re.M)
        title, body = markdown_article(material_blocks(raw), slug_title(file))
        route = key.removesuffix(".md")
        route = route.removesuffix("/index") if route != "index" else ""
        pack.add_markdown(key, title, body, pack.source_url + (route + "/" if route else ""),
                          (file.stem, "FastAPI " + title, *aliases), file_assets(file, base))
    return pack


def matches_moniker(expression, family, version=(10, 0)):
    def matches_all(group):
        terms = list(re.finditer(r"(>=|<=|=|>|<)?\s*([\w]+)-(\d+(?:\.\d+)+)", group))
        if not terms or re.sub(r"(>=|<=|=|>|<)?\s*[\w]+-\d+(?:\.\d+)+", "", group).strip():
            return None
        for term in terms:
            operator, tag, number = term.groups()
            if tag != family:
                return None
            wanted = tuple(map(int, number.split(".")))
            if not {None: version == wanted, "=": version == wanted, ">": version > wanted,
                    "<": version < wanted, ">=": version >= wanted, "<=": version <= wanted}[operator]:
                return False
        return True

    results = [matches_all(group) for group in expression.split("||")]
    return None if None in results else any(results)


def docfx_monikers(raw, family):
    result = []
    visibility = [True]
    code = None
    for line in raw.splitlines():
        fence = re.match(r"^\s*(`{3,}|~{3,})", line)
        if fence:
            marker = fence[1]
            if code is None:
                code = marker
            elif marker.startswith(code) and not line.strip().strip(code[0]):
                code = None
            if visibility[-1]:
                result.append(line)
            continue
        if code:
            if visibility[-1]:
                result.append(line)
            continue
        start = re.fullmatch(r'\s*:{3,}\s*moniker\s+range="([^"]+)"\s*', line)
        if start:
            match = matches_moniker(start[1], family)
            visibility.append(visibility[-1] and match is not False)
            if visibility[-1] and match is None:
                result.extend(("", f"> **Version range:** {start[1]}", ""))
        elif re.fullmatch(r"\s*:{3,}\s*moniker-end\s*", line) and len(visibility) > 1:
            visibility.pop()
        elif visibility[-1]:
            result.append(line)
    return "\n".join(result) + "\n"


def docfx_target(target):
    """Remove a Markdown link title and normalize DocFX's path separators."""
    target = re.sub(r'''\s+(?:"[^"]*"|'[^']*')\s*$''', "", target.strip())
    return target.removeprefix("<").removesuffix(">").replace("\\ ", " ").replace("\\", "/")


def docfx_region(content, wanted):
    markers = re.finditer(
        r'''(?mi)^[ \t]*(?:\#[ \t]*(?P<region>region|end[ \t]*region)\b[ \t]*(?P<label>[^\n]*)'''
        r'''|(?://|'|<!--|@\*|/\*)[ \t]*<[ \t]*(?P<close>/)?[ \t]*(?P<name>[\w.-]+)[ \t]*>[^\n]*)$''',
        content,
    )

    def name_key(name):
        name = name.strip(" \"'").casefold()
        return re.sub(r"^snippet[_-]?(?=\w)", "", name)

    wanted = name_key(wanted)
    start = None
    depth = 0
    for marker in markers:
        region, label, close, name = marker.groups()
        if region:
            opening = region.casefold() == "region"
            if depth:
                depth += 1 if opening else -1
                if not depth:
                    return content[start:marker.start()]
            elif start is None and opening and name_key(label) == wanted:
                start = marker.end() + 1
                depth = 1
        elif not depth and name_key(name) == wanted:
            if close and start is not None:
                return content[start:marker.start()]
            if not close and start is None:
                start = marker.end() + 1
    return None


def docfx_includes(raw, file, base, repo, pack, parents=()):
    roots = [repo.resolve()]
    if pack.id == "aspnet":
        roots.append((repo.parent / "AspNetCore.Docs.Samples").resolve())

    def source_path(target):
        url = urlsplit(target)
        target = unquote(url.path)
        if url.scheme or url.netloc or target.startswith("/"):
            return None
        rooted = target.removeprefix("~/").lstrip("/")
        candidates = ((base / rooted, repo / rooted) if target.startswith("~/")
                      else (file.parent / target,))
        for candidate in candidates:
            path = candidate.resolve()
            if any(path.is_relative_to(root) for root in roots) and path.is_file():
                return path
        return None

    def missing(target, label, reason):
        article = (parents[0] if parents else file).relative_to(repo)
        origin = file.relative_to(repo)
        location = f" (via {origin})" if origin != article else ""
        pack.missing.add(f"{article}: {reason}{location}: {target}")
        name, commit = ("AspNetCore.Docs", ASPNET_COMMIT) if pack.id == "aspnet" else ("docs", DOTNET_COMMIT)
        official = f"https://github.com/dotnet/{name}/blob/{commit}/{quote(article.as_posix(), safe='/')}"
        return f"\n> {label} was unavailable in this snapshot. [View the official source]({official}).\n"

    def include(match):
        target = docfx_target(match[1])
        path = source_path(target)
        if not path:
            return missing(target, "Included content", "file absent")
        if path == file or path in parents:
            return missing(target, "Included content", "cyclic include")
        if path.suffix != ".md" or len(parents) >= 12:
            return missing(target, "Included content", "unsupported include")
        included = without_front_matter(read_text(path))
        return docfx_includes(included, path, base, repo, pack, (*parents, file))

    def snippet(language, target, region=""):
        target = docfx_target(target)
        path = source_path(target)
        parts = urlsplit(target)
        options = parse_qs(parts.query)
        region = region or options.get("name", [""])[0] or unquote(parts.fragment)
        reference = target + (f" [{region}]" if region else "")
        if not path:
            return missing(reference, "Code sample", "file absent")
        if path.stat().st_size > 8_000_000:
            return missing(reference, "Code sample", "source exceeds size limit")
        content = read_text(path)
        if region:
            content = docfx_region(content, region)
            if content is None:
                return missing(reference, "Code sample region", "region absent")
        ranges = options.get("range", [""])[0]
        if ranges:
            lines = content.splitlines()
            selected = []
            for segment in ranges.split(","):
                match = re.fullmatch(r"(\d+)(?:-(\d+))?", segment.strip())
                if match:
                    selected.extend(lines[int(match[1]) - 1:int(match[2] or match[1])])
            if selected:
                content = "\n".join(selected)
        if len(content) > 500_000:
            return missing(reference, "Code sample", "excerpt exceeds size limit")
        return code_fence(textwrap.dedent(content), language)

    def quoted_markup(match, markup):
        prefix = match["quote"].rstrip()
        if not prefix:
            return markup
        return "\n" + "\n".join(prefix + (" " + line if line else "") for line in markup.strip("\n").splitlines()) + "\n"

    def directive_attributes(match):
        return dict(re.findall(r'([\w-]+)[ \t]*=[ \t]*"([^"]*)"', match["attributes"]))

    def code_block(match):
        attrs = directive_attributes(match)
        if not attrs.get("source"):
            return missing(match[0], "Code sample", "missing source attribute")
        markup = snippet(attrs.get("language", ""), attrs["source"], attrs.get("id", ""))
        trailer = match["trailer"].strip()
        if trailer and trailer not in markup:
            markup += "\n" + trailer + "\n"
        return quoted_markup(match, markup)

    def image_block(match):
        attrs = directive_attributes(match)
        if not attrs.get("source"):
            return ""
        markup = '\n<img src="' + html.escape(attrs["source"], quote=True) + '" alt="' + html.escape(
            attrs.get("alt-text", ""), quote=True) + '">\n'
        return quoted_markup(match, markup)

    def zone(match):
        pivot = directive_attributes(match).get("pivot", "").strip()
        if not pivot:
            return match[0]
        label = pivot.replace("-", " ")
        return f"\n**{label[:1].upper() + label[1:]}**\n"

    def expand_directives(text):
        prefix = r'^(?P<quote>(?:[ \t]*>[ \t]*)*)[ \t]*:::[ \t]*'
        attributes = r'(?P<attributes>(?:\s+[\w-]+[ \t]*=[ \t]*"[^"\n]*")*)'
        text = re.sub(prefix + r'image\b' + attributes + r'(?:[ \t]*:::)?[ \t]*$', image_block, text, flags=re.M)
        text = re.sub(r'^[ \t]*:::[ \t]*zone\b' + attributes + r'[ \t]*(?::::)?[ \t]*$', zone, text, flags=re.M)
        text = re.sub(r'^[ \t]*:::[ \t]*(?:zone-end|column(?:-end)?|row(?:-end)?)\b[^\n]*$', "", text, flags=re.M)
        text = re.sub(r':::[ \t]*no-loc[ \t]+text="([^\n]*?)"[ \t]*:::', r"\1", text)
        text = re.sub(r"<xref:([^>\s]+)[ \t]*>", r"[`\1`](xref:\1)", text)
        text = re.sub(prefix + r'code\b' + attributes + r'\s*:::(?P<trailer>[^\n]*)$', code_block, text, flags=re.M)
        return transform_prose(text, lambda prose: re.sub(
            r"\[!code-([\w+-]+)\s*\[[^\]]*\]\(([^)]+)\)\]",
            lambda match: snippet(*match.groups()), prose, flags=re.I))

    def expand(text):
        text = re.sub(r"\[!INCLUDE\s*\[[^\]]*\]\(([^)]+)\)\]", include, text, flags=re.I)
        return transform_prose(text, expand_directives)

    family = "aspnetcore" if pack.id == "aspnet" else "net"
    raw = transform_prose(raw, lambda text: re.sub(r"<!--.*?-->", "", text, flags=re.S))
    raw = docfx_monikers(raw, family)
    return transform_prose(raw, expand)


def transform_prose(raw, transform):
    sections = re.split(r"(^[ \t]*(`{3,}|~{3,})[^\n]*\n.*?^[ \t]*\2[`~]*[ \t]*$)",
                        raw, flags=re.M | re.S)
    return "".join(transform(section) if index % 3 == 0 else section
                   for index, section in enumerate(sections) if index % 3 != 2)


def microsoft_pack(cache, scratch, *, aspnet=False):
    name = "AspNetCore.Docs" if aspnet else "docs"
    commit = ASPNET_COMMIT if aspnet else DOTNET_COMMIT
    archive = "aspnet-docs-full.tar.gz" if aspnet else "dotnet-docs-full.tar.gz"
    microsoft = scratch / "microsoft"
    safe_extract_tar(cache / archive, microsoft, lambda path: True)
    repo = microsoft / f"{name}-{commit}"
    source_detail = f"Microsoft documentation repository, pinned commit [{commit}](https://github.com/dotnet/{name}/tree/{commit})."
    if aspnet:
        safe_extract_tar(cache / ASPNET_SAMPLES_ARCHIVE, microsoft, lambda path: True)
        samples = microsoft / f"AspNetCore.Docs.Samples-{ASPNET_SAMPLES_COMMIT}"
        samples.rename(microsoft / "AspNetCore.Docs.Samples")
        pack = Pack("aspnet", "ASP.NET Core", "dotnet", "2026.11.06",
                    "Microsoft Learn ASP.NET Core fundamentals, Web APIs and tutorials snapshot.",
                    "https://learn.microsoft.com/en-us/aspnet/core/", "CC-BY-4.0; accompanying code MIT",
                    source_detail + "\n\nASP.NET Core 10.0 moniker. Samples repository commit: "
                    f"https://github.com/dotnet/AspNetCore.Docs.Samples/tree/{ASPNET_SAMPLES_COMMIT}")
        base = repo / "aspnetcore"
    else:
        pack = Pack("csharp", "C# and .NET", "dotnet", "2026.11.06",
                    "Microsoft Learn C# language, runtime, tooling and .NET fundamentals snapshot.",
                    "https://learn.microsoft.com/en-us/dotnet/", "CC-BY-4.0; accompanying code MIT",
                    source_detail + "\n\n.NET 10.0 moniker where upstream specifies a .NET version range.")
        base = repo / "docs"
    pack.notice(repo / "LICENSE")
    pack.notice(repo / "LICENSE-CODE")
    if aspnet:
        samples = microsoft / "AspNetCore.Docs.Samples"
        pack.notice(samples / "LICENSE", "SAMPLES-LICENSE")
        pack.notice(samples / "LICENSE-CODE", "SAMPLES-LICENSE-CODE")
    pages = []
    groups = {"fundamentals", "host-and-deploy", "performance", "security", "test", "tutorials", "web-api"}
    if not aspnet:
        groups = {"core", "csharp", "fundamentals", "standard"}
    for file in sorted(base.rglob("*.md")):
        relative = file.relative_to(base)
        key = relative.as_posix()
        if len(relative.parts) > 1 and relative.parts[0] not in groups:
            continue
        if len(relative.parts) == 1 and file.stem not in {"get-started", "overview", "welcome"}:
            continue
        if any(part in {"includes", "_includes", "snippets", "media", "samples"}
               for part in relative.parts) or file.stem.lower() in {
                   "agents", "contributing", "readme", "security", "code_of_conduct", "pull_request_template"
               }:
            continue
        raw = read_text(file)
        if re.search(r"^redirect_url:", raw, re.M):
            continue
        url = pack.source_url + key.removesuffix(".md")
        uid = re.search(r"^uid:\s*([^\n]+)", raw, re.M)
        if uid:
            pack.xrefs[uid[1].strip().strip("\"'")] = url
        pages.append((file, key, raw, url))
    for file, key, raw, url in pages:
        title, body = markdown_article(docfx_includes(raw, file, base, repo, pack), slug_title(file))
        pack.add_markdown(key, title, body, url, (file.stem, "ASP.NET " + title if aspnet else "C# " + title),
                          file_assets(file, base))
    return pack


GO_PACKAGES = """archive/tar archive/zip bufio bytes cmp compress/gzip context crypto/aes
crypto/hmac crypto/rand crypto/sha256 crypto/tls database/sql embed encoding/base64
encoding/csv encoding/gob encoding/hex encoding/json encoding/xml errors flag fmt
hash html/template io io/fs iter log log/slog maps math math/rand/v2 mime/multipart
net net/http net/http/httptest net/netip net/url os os/exec os/signal path path/filepath
reflect regexp runtime slices sort strconv strings sync sync/atomic testing text/template
time unicode unicode/utf8""".split()


GO_DECLARATION = re.compile(r"^(?:func\s+(?:\(([^)]+)\)\s+)?|(?:type|var|const)\s+)([A-Z]\w*)")


def go_article(pack, key, title, body, source, aliases):
    html_nodes = []
    prose = []
    code = []
    declarations = False

    def flush():
        if prose:
            html_nodes.append(Node("p", children=[" ".join(prose)]))
            prose.clear()
        if code:
            html_nodes.append(Node("pre", children=[Node("code", {"class": "language-go"}, ["\n".join(code)])]))
            code.clear()

    for line in body.splitlines():
        if line in {"CONSTANTS", "VARIABLES", "FUNCTIONS", "TYPES"}:
            flush()
            declarations = True
            html_nodes.append(Node("h2", {"id": line.lower()}, [line.title()]))
        elif line.startswith(("# ", "    # ")):
            flush()
            heading = line.strip()[2:]
            html_nodes.append(Node("h2", children=[heading]))
        elif match := GO_DECLARATION.match(line):
            flush()
            receiver, symbol = match.groups()
            anchor = symbol if not receiver else receiver.split()[-1].lstrip("*").partition("[")[0] + "." + symbol
            html_nodes.append(Node("h3", {"id": anchor}, [anchor]))
            aliases.append(f"{title.split('/')[-1]}.{anchor}")
            code.append(line)
        elif not line.strip():
            if code and code[-1] != "":
                code.append("")
            if prose:
                flush()
        elif line.startswith("    ") and not declarations or line.startswith(("        ", "    \t")):
            if prose:
                flush()
            code.append(line[4:] if not declarations else line[8:] if line.startswith("        ") else line[5:])
        elif line.startswith("    "):
            if code:
                while code and not code[-1]:
                    code.pop()
                flush()
            prose.append(line.strip())
        elif line.startswith(("package ", "func ", "const ", "var ", "type ", "\t")) or line in {")", "}"}:
            if prose:
                flush()
            code.append(line)
        else:
            if code:
                while code and not code[-1]:
                    code.pop()
                flush()
            prose.append(line.strip())
    flush()
    pack.add(key, title, clean_text(body), source, aliases, render_html(Node("", children=html_nodes)))


def go_packs(goroot, chi_root, scratch):
    go = goroot / "bin/go"
    env = {**os.environ, "GOTOOLCHAIN": "local", "GOTELEMETRY": "off",
           "GOWORK": "off", "GOPROXY": "off", "GOCACHE": str(scratch / "go-build")}
    version = (goroot / "VERSION").read_text().splitlines()[0].removeprefix("go")
    pack = Pack("go", "Go", "go", version, "Official language specification and common standard-library APIs.",
                "https://go.dev/", "BSD-3-Clause",
                "The Go Authors, extracted from the Go " + version + " distribution and go doc.")
    pack.notice(goroot / "LICENSE")
    pack.notice(goroot / "PATENTS")
    for file, uri in (("go_spec.html", "ref/spec"), ("go_mem.html", "ref/mem"), ("asm.html", "doc/asm")):
        html_page(pack, read_text(goroot / "doc" / file), "doc/" + file,
                  source=pack.source_url + uri, aliases=(uri.split("/")[-1],))
    for package in GO_PACKAGES:
        body = command(go, "doc", "-all", package, env=env, cwd=goroot)
        aliases = [package, "go " + package]
        # Package-level declarations supplement exact matching for frequent API symbols.
        aliases.extend(f"{package.split('/')[-1]}.{item}" for item in re.findall(
            r"^(?:type|func) ([A-Z]\w*)[ (\[]", body, flags=re.M))
        go_article(pack, "std/" + package, package, body, f"https://pkg.go.dev/{package}@go{version}", aliases)

    mod = "github.com/go-chi/chi/v5"
    version = chi_root.name.partition("@v")[2]
    chi = Pack("chi", "Chi", "go", version, "Official Chi router, middleware and example documentation.",
               f"https://pkg.go.dev/{mod}@v{version}", "MIT",
               f"The go-chi contributors, module {mod} v{version}. https://github.com/go-chi/chi/tree/v{version}")
    chi.notice(chi_root / "LICENSE")
    for package in (".", "./middleware"):
        body = command(go, "doc", "-all", package, env=env, cwd=chi_root)
        tail = package.removeprefix(".")
        aliases = [mod + tail, "chi" + tail]
        aliases.extend("chi." + item for item in re.findall(r"^(?:type|func) ([A-Z]\w*)[ (\[]", body, flags=re.M))
        go_article(chi, "api" + tail, "Chi " + (tail.removeprefix("/") or "Router"), body,
                   f"https://pkg.go.dev/{mod}@v{version}{tail}", aliases)
    for file in [chi_root / "README.md", *sorted((chi_root / "_examples").glob("*/main.go"))]:
        if not file.is_file():
            continue
        rel = file.relative_to(chi_root).as_posix()
        source = f"https://github.com/go-chi/chi/blob/v{version}/{rel}"
        if file.suffix == ".go":
            title, body = "Chi example: " + file.parent.name.replace("-", " "), read_text(file)
            chi.add(rel, title, body, source, (file.parent.name,),
                    render_html(Node("pre", children=[Node("code", {"class": "language-go"}, [body])])))
        else:
            title, body = markdown_article(read_text(file), "Chi README")
            chi.add_markdown(rel, title, body, source, (file.parent.name,))
    return pack, chi


def java_pack(java_home, scratch):
    version = "26.0.2.1"
    if f'JAVA_VERSION="{version}"' not in read_text(java_home / "release").splitlines():
        raise RuntimeError(f"Java pack requires JDK {version}: {java_home}")
    pack = Pack("java", "Java SE", "java", version,
                "Official OpenJDK API reference generated from curated JDK 26 library sources.",
                "https://docs.oracle.com/en/java/javase/26/docs/api/", "GPL-2.0 with Classpath Exception",
                "OpenJDK contributors, Javadoc generated from the Liberica JDK 26.0.2.1 src.zip. "
                "The upstream links point to the corresponding Java SE reference pages. "
                "The exact [Java source subset](openjdk-26.0.2.1-javadoc-sources.zip) is included.")
    for name in ("LICENSE", "ADDITIONAL_LICENSE_INFO", "ASSEMBLY_EXCEPTION"):
        pack.notice(java_home / "legal/java.base" / name)
    packages = """java.lang java.lang.annotation java.lang.constant java.lang.invoke
java.lang.management java.lang.module java.lang.ref java.lang.reflect java.math java.io
java.net java.net.http java.nio java.nio.channels java.nio.charset java.nio.file
java.nio.file.attribute java.security java.sql java.text java.time java.time.format
java.time.temporal java.time.zone java.util java.util.concurrent java.util.concurrent.atomic
java.util.concurrent.locks java.util.function java.util.logging java.util.regex java.util.stream
javax.crypto javax.net javax.net.ssl javax.sql javax.xml.parsers""".split()
    source_root = scratch / "jdk-source"
    with zipfile.ZipFile(java_home / "lib/src.zip") as archive:
        for name in archive.namelist():
            source = Path(name)
            if source.suffix != ".java" or source.is_absolute() or ".." in source.parts:
                continue
            module, *parts, _ = source.parts
            if ".".join(parts).removesuffix(".snippet-files") not in packages or source.name == "module-info.java":
                continue
            target = source_root / source
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(archive.read(name))
    source_archive = scratch / f"openjdk-{version}-javadoc-sources.zip"
    with zipfile.ZipFile(source_archive, "w") as archive:
        for source in sorted(source_root.rglob("*.java")):
            archive.writestr(zipfile.ZipInfo(source.relative_to(source_root).as_posix()), source.read_bytes(),
                             compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
        for name in pack.notices:
            archive.writestr(zipfile.ZipInfo(name), (java_home / "legal/java.base" / name).read_bytes())
        archive.writestr(zipfile.ZipInfo("README.txt"),
                         f"OpenJDK source and snippets from BellSoft Liberica {version} lib/src.zip.\n"
                         "The source files are preserved byte for byte. Devline generates the API "
                         "reference with scripts/fetch-docs.py.\n")
    pack.attachments[source_archive.name] = source_archive
    for module in sorted(source_root.iterdir()):
        target = scratch / "jdk-html" / module.name
        # Run one module at a time. Patch-module lets Javadoc resolve core Java packages
        # against the same installed JDK without including proprietary Oracle HTML.
        snippet_paths = os.pathsep.join(str(path) for path in sorted(module.rglob("snippet-files")))
        snippets = ["--snippet-path", snippet_paths] if snippet_paths else []
        command(java_home / "bin/javadoc", "-quiet", "-Xdoclint:none", "--patch-module",
                f"{module.name}={module}", "-d", target, *snippets,
                "-tag", "apiNote:a:API Note:", "-tag", "implSpec:a:Implementation Requirements:",
                "-tag", "implNote:a:Implementation Note:",
                *sorted(file for file in module.rglob("*.java") if "snippet-files" not in file.parts))
        module_output = target / module.name
        for file in sorted(module_output.rglob("*.html")):
            if file.stem in {"package-tree", "module-summary"}:
                continue
            rel = file.relative_to(target).as_posix()
            pkg = ".".join(file.relative_to(module_output).parts[:-1])
            aliases = (pkg,) if file.stem == "package-summary" else (f"{pkg}.{file.stem}", file.stem)
            html_page(pack, read_text(file), rel, mode="javadoc", aliases=aliases, assets=file_assets(file, target))
    return pack


def spring_pack(reference, cache):
    pack = Pack("spring-boot", "Spring Boot", "java", "4.1.2",
                "Official Spring Boot reference, how-to guides and tutorials.",
                "https://docs.spring.io/spring-boot/4.1/", "Apache-2.0",
                "Spring Boot contributors, Antora documentation for 4.1.2. https://github.com/spring-projects/spring-boot/tree/v4.1.2")
    pack.notice(cache / "spring-LICENSE", "LICENSE")
    base = reference / "spring-boot"
    groups = {"reference", "how-to", "tutorial", "appendix", "maven-plugin", "gradle-plugin"}
    for file in sorted(base.rglob("*.html")):
        rel = file.relative_to(base)
        if len(rel.parts) > 1 and rel.parts[0] not in groups:
            continue
        html_page(pack, read_text(file), rel.as_posix(), mode="spring", aliases=(file.stem,),
                  title_prefix="Spring Boot · ", assets=file_assets(file, base))
    return pack


RUBY_CLASSES = """Array BasicObject Binding Class Comparable Dir Encoding Enumerable Enumerator
Enumerator::Lazy ENV Exception FalseClass Fiber File File::Stat FileTest Float GC Hash Integer
IO Kernel MatchData Method Module NilClass Numeric Object ObjectSpace Proc Process Queue Random
Range Rational Regexp Ractor Set Signal String Struct Symbol Thread Time TrueClass
CSV Date DateTime Digest FileUtils Find JSON Logger Net::HTTP Open3 OpenSSL OptionParser
Pathname PP Psych Socket StringIO Tempfile URI YAML""".split()


def ruby_pack(cache, scratch, ruby):
    version = command(ruby, "-e", "print RUBY_VERSION")
    if version != "4.0.9":
        raise RuntimeError(f"Ruby documentation requires installed Ruby 4.0.9, found {version}")
    root = scratch / ("ruby-" + version)
    safe_extract_tar(cache / "ruby.tar.gz", scratch, lambda name:
                     name.name in {"COPYING", "COPYING.ja", "BSDL", "LEGAL"} or str(name).partition("/")[2].startswith("doc/"))
    pack = Pack("ruby", "Ruby", "ruby", version,
                "Official Ruby language guides and core/standard-library reference from RDoc.",
                "https://docs.ruby-lang.org/en/4.0/", "Ruby License or BSD-2-Clause",
                "The Ruby contributors, ruby-4.0.9 source and installed ri documentation database.")
    for file in ("COPYING", "COPYING.ja", "BSDL", "LEGAL"):
        pack.notice(root / file)
    guides = [
        file for file in sorted((root / "doc").rglob("*"))
        if file.suffix in {".rdoc", ".md"} and len(file.relative_to(root / "doc").parts) <= 2
        and not file.stem.startswith(("NEWS", "ChangeLog", "contributing", "maintainers"))
        and not file.stem.endswith(".ja")
    ]
    rendered = scratch / "ruby-html"
    command(ruby.with_name("rdoc"), "--quiet", "--op", rendered,
            *[file.relative_to(root) for file in guides if file.suffix == ".rdoc"], cwd=root)
    for file in guides:
        key = file.relative_to(root / "doc").as_posix()
        suffix = "_" + file.suffix.removeprefix(".") + ".html"
        route = "doc/" + key.removesuffix(file.suffix) + suffix
        source = pack.source_url + route
        if file.suffix == ".rdoc":
            generated = rendered / route
            if generated.is_file():
                html_page(pack, read_text(generated), "doc/" + key, source=source, aliases=(file.stem,),
                          assets=file_assets(generated, rendered))
            else:
                pack.missing.add(f"{key}: RDoc output for {route}")
        else:
            title, body = markdown_article(read_text(file), slug_title(file))
            pack.add_markdown("doc/" + key, title, body, source, (file.stem,), file_assets(file, root))
    ri = ruby.with_name("ri")
    for cls in RUBY_CLASSES:
        body = command(ri, "--no-pager", "--format=markdown", "--no-gems", cls, check=False)
        if body:
            url = pack.source_url + cls.replace("::", "/") + ".html"
            pack.add_markdown("api/" + cls, cls, body, url, (cls,))
    for symbol in """Array#map Array#select Array#filter Array#each Array#each_with_index Array#reduce
Array#sum Hash#fetch Hash#transform_keys Hash#transform_values String#split String#match String#encode
String#strip String#scan String#gsub String#sub Enumerable#map Enumerable#reduce Enumerable#filter_map
Enumerable#group_by Enumerator::Lazy#map File.read File.open File.write IO.popen Kernel#require
Kernel#puts Kernel#sprintf Proc#call Regexp#match Thread#join Time.now Time.at JSON.parse
JSON.generate URI.parse Net::HTTP.get CSV.foreach Pathname#join OptionParser#parse!""".split():
        body = command(ri, "--no-pager", "--format=markdown", "--no-gems", symbol, check=False)
        if body:
            prefix, mark, method = symbol.partition("#")
            if not mark:
                prefix, _, method = symbol.partition(".")
            anchor = "#method-" + ("i" if mark else "c") + "-" + method.replace("!", "-21").replace("?", "-3F")
            pack.add_markdown("method/" + symbol, symbol, body,
                              pack.source_url + prefix.replace("::", "/") + ".html" + anchor, (symbol,))
    return pack


def rails_pack(cache, scratch, rust_docs):
    pack = Pack("rails", "Ruby on Rails", "ruby", "8.1.3",
                "Official Rails guides for routing, controllers, views, models, jobs and deployment.",
                "https://guides.rubyonrails.org/v8.1/", "CC-BY-SA-4.0",
                "Ruby on Rails contributors, guides/source at tag v8.1.3. "
                "Adapted as searchable text under the same CC BY-SA 4.0 license.")
    pack.notice(rust_docs.parent / "licenses/CC-BY-SA-4.0.txt", "LICENSE")
    safe_extract_tar(cache / "rails.tar.gz", scratch, lambda name:
                     str(name).partition("/")[2].startswith("guides/source/") and name.suffix == ".md"
                     or str(name).partition("/")[2].startswith("guides/assets/images/") and name.suffix in IMAGE_SUFFIXES)
    guides = scratch / "rails-8.1.3/guides"
    base = guides / "source"
    for file in sorted(base.glob("*.md")):
        if re.match(r"^[0-9]|^upgrading", file.name):
            continue
        raw = transform_prose(read_text(file), lambda prose: re.sub(
            r"^(TIP|NOTE|WARNING|IMPORTANT|CAUTION): (.+)$", r"> [!\1]\n> \2", prose, flags=re.M))
        title, body = markdown_article(raw, slug_title(file))
        pack.add_markdown(file.name, title, body, pack.source_url + file.stem + ".html",
                          (file.stem, "Rails " + title), file_assets(file, guides, guides / "assets"))
    return pack


def rust_pack(rust_docs):
    pack = Pack("rust", "Rust", "rust", "1.99.1",
                "The Rust Book, Reference, Cargo Book, and selected standard-library APIs.",
                "https://doc.rust-lang.org/1.99.1/", "MIT OR Apache-2.0",
                "The Rust project developers, Rust 1.99.1 toolchain documentation. "
                "Standard-library selection focuses on commonly used modules and symbols.")
    for file in ("MIT.txt", "Apache-2.0.txt"):
        pack.notice(rust_docs.parent / "licenses" / file)
    pack.notice(rust_docs.parent / "COPYRIGHT.html", "COPYRIGHT.html")
    for name in ("book", "reference", "cargo", "edition-guide"):
        for file in sorted((rust_docs / name).rglob("*.html")):
            if file.stem in {"print", "404", "toc", "theme", "search"}:
                continue
            rel = file.relative_to(rust_docs).as_posix()
            html_page(pack, read_text(file), rel, aliases=(file.stem,), assets=file_assets(file, rust_docs))
    std = rust_docs / "std"
    modules = """any borrow boxed cell char clone cmp collections convert default env error ffi
fmt fs future hash hint io iter marker mem net num ops option os panic path pin primitive
process ptr rc result slice str string sync task thread time vec""".split()
    for file in sorted(std.rglob("*.html")):
        relative = file.relative_to(std)
        if len(relative.parts) > 3 or relative.parts[0] not in modules and len(relative.parts) > 1:
            continue
        if not file.stem.startswith(("struct.", "trait.", "enum.", "fn.", "primitive.", "macro.", "type.")) and file.stem != "index":
            continue
        symbol = file.stem.split(".")[-1]
        if file.stem == "index":
            qualified = "::".join(("std", *relative.parts[:-1]))
            aliases = (qualified,)
        else:
            qualified = "::".join(("std", *relative.parts[:-1], symbol))
            aliases = (qualified, symbol)
        html_page(pack, read_text(file), "std/" + relative.as_posix(), aliases=aliases, assets=file_assets(file, rust_docs))
    return pack


def axum_pack(reference, crate):
    pack = Pack("axum", "Axum", "rust", "0.8.9",
                "Official Rustdoc for Axum routing, extractors, responses and middleware.",
                "https://docs.rs/axum/0.8.9/", "MIT",
                "Tokio contributors, Rustdoc from the installed axum 0.8.9 crate.")
    pack.notice(crate / "LICENSE")
    base = reference / "rust-crates/docs"
    for file in sorted((base / "axum").rglob("*.html")):
        key = file.relative_to(base).as_posix()
        symbol = file.stem.split(".")[-1]
        parts = file.relative_to(base).parts
        alias = "::".join(parts[:-1]) if symbol == "index" else "::".join((*parts[:-1], symbol))
        aliases = [alias, symbol]
        if symbol in {"Router", "Json", "Form", "Extension", "serve", "ServiceExt"}:
            aliases.append(f"axum::{symbol}")
        html_page(pack, read_text(file), key, aliases=aliases, title_prefix="Axum · ", assets=file_assets(file, base))
    return pack


def hex_pack(cache, scratch, reference, name):
    labels = {
        "elixir": ("Elixir", "1.20.5", "Apache-2.0", "Official language guides and core/standard library documentation."),
        "phoenix": ("Phoenix", "1.8.16", "MIT", "Official Phoenix guides and API documentation."),
        "ecto": ("Ecto", "3.14.3", "Apache-2.0", "Official Ecto database library guides and API documentation."),
    }
    title, version, license_name, description = labels[name]
    url = f"https://hexdocs.pm/{name}/{version}/"
    pack = Pack(name, title, "elixir", version, description, url, license_name,
                f"{title} project contributors, ExDoc package {name}-{version} from HexDocs.")
    pack.notice(cache / f"{name}-LICENSE", "LICENSE")
    if name == "elixir":
        base = scratch / "elixir"
        safe_extract_tar(cache / "elixir-docs.tar.gz", base, lambda file: file.suffix in {".html", *IMAGE_SUFFIXES})
    else:
        base = reference / "hexdocs" / name
    for file in sorted(base.glob("*.html")):
        if file.stem in {"404", "api-reference", "search", "readme", "changelog"}:
            continue
        html_page(pack, read_text(file), file.name, mode="hex", aliases=(file.stem,), assets=file_assets(file, base))
    return pack


def download(cache, name):
    path = cache / name
    if path.is_file():
        return
    url = DOWNLOADS[name]
    request = Request(url, headers={"User-Agent": "DevKit Docs Builder"})
    partial = path.with_suffix(path.suffix + ".part")
    with urlopen(request, timeout=120) as response, partial.open("wb") as output:
        shutil.copyfileobj(response, output)
    partial.replace(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, required=True, help="directory containing the upstream archives and licenses")
    parser.add_argument("--reference", type=Path, required=True, help="prepared Offline Developer Kit/reference tree")
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "data")
    parser.add_argument("--fetch", action="store_true", help="download any missing upstream archives and licenses into --cache")
    parser.add_argument("--only", nargs="+", help="rebuild selected pack IDs and preserve other catalog entries")
    parser.add_argument("--go-root", type=Path)
    parser.add_argument("--chi-root", type=Path)
    parser.add_argument("--rust-docs", type=Path)
    parser.add_argument("--axum-crate", type=Path)
    parser.add_argument("--java-home", type=Path)
    parser.add_argument("--ruby", type=Path)
    parser.add_argument("--starlette-wheel", type=Path, default=Path(__file__).resolve().parents[1]
                        / "data/templates/python/fastapi-api/1.1.0/project/wheelhouse/starlette-1.10.0-py3-none-any.whl")
    options = parser.parse_args()
    cache, reference, data = (path.resolve() for path in (options.cache, options.reference, options.output))
    cache.mkdir(parents=True, exist_ok=True)
    data.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="devkit-docs-", dir=cache) as tmp:
        scratch = Path(tmp)
        def go_sources():
            goroot = options.go_root or Path(command("go", "env", "GOROOT").strip())
            chi = options.chi_root or Path(command("go", "env", "GOMODCACHE").strip()) / "github.com/go-chi/chi/v5@v5.2.4"
            return go_packs(goroot, chi, scratch)

        def java_source():
            home = options.java_home or Path(command("/usr/libexec/java_home").strip())
            return java_pack(home, scratch)

        def rust_docs_source():
            return options.rust_docs or Path(command("rustup", "doc", "--path").strip()).parent

        def axum_source():
            registry = Path.home() / ".cargo/registry/src"
            crate = options.axum_crate or next(registry.glob("*/axum-0.8.9"), None)
            if crate is None:
                raise RuntimeError("axum 0.8.9 crate missing. Supply --axum-crate.")
            return axum_pack(reference, crate)

        builders = {
            "python": (("python.zip", "python-LICENSE"), lambda: python_pack(cache)),
            "fastapi": (("fastapi-0.162.0.tar.gz",), lambda: fastapi_pack(cache, scratch, options.starlette_wheel.resolve())),
            "csharp": (("dotnet-docs-full.tar.gz",), lambda: microsoft_pack(cache, scratch)),
            "aspnet": (("aspnet-docs-full.tar.gz", ASPNET_SAMPLES_ARCHIVE),
                       lambda: microsoft_pack(cache, scratch, aspnet=True)),
            "go": ((), go_sources),
            "chi": ((), go_sources),
            "java": ((), java_source),
            "spring-boot": (("spring-LICENSE",), lambda: spring_pack(reference, cache)),
            "ruby": (("ruby.tar.gz",), lambda: ruby_pack(cache, scratch, options.ruby or Path(shutil.which("ruby")).resolve())),
            "rails": (("rails.tar.gz",), lambda: rails_pack(cache, scratch, rust_docs_source())),
            "rust": ((), lambda: rust_pack(rust_docs_source())),
            "axum": ((), axum_source),
            "elixir": (("elixir-docs.tar.gz", "elixir-LICENSE"), lambda: hex_pack(cache, scratch, reference, "elixir")),
            "phoenix": (("phoenix-LICENSE",), lambda: hex_pack(cache, scratch, reference, "phoenix")),
            "ecto": (("ecto-LICENSE",), lambda: hex_pack(cache, scratch, reference, "ecto")),
        }
        selected = set(options.only or builders)
        if not selected <= builders.keys():
            parser.error(f"unknown pack IDs: {sorted(selected - builders.keys())}")
        if options.fetch:
            names = set(name for key in selected for name in builders[key][0])
            with ThreadPoolExecutor(max_workers=4) as pool:
                list(pool.map(lambda name: download(cache, name), sorted(names)))
        catalog_path = data / "docs-catalog.json"
        entries = {item["id"]: item for item in json.loads(read_text(catalog_path))} if options.only and catalog_path.is_file() else {}
        done = set()
        for key, (_, build) in builders.items():
            if key not in selected or key in done:
                continue
            result = build()
            packs = result if isinstance(result, tuple) else (result,)
            for pack in packs:
                if pack.id in selected:
                    entries[pack.id] = pack.emit(data)
                    done.add(pack.id)
                    atomic_json(catalog_path, [entries[id] for id in builders if id in entries])


if __name__ == "__main__":
    main()
