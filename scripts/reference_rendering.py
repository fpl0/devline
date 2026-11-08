"""Render publisher articles as sanitized offline HTML and searchable text."""

import base64
from dataclasses import dataclass, field
import html
from html.parser import HTMLParser
from pathlib import Path
import posixpath
import re
import subprocess
import textwrap
from urllib.parse import quote, unquote, urljoin, urlsplit
import xml.etree.ElementTree as ET


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
PANDOC_VERSION = "3.13"
MAX_IMAGE_BYTES = 750_000
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
        github_source = pack_id == "infrastructure" and urlsplit(source).hostname == "github.com"
        if not github_source and not target.scheme and target.path.endswith((".md", ".mdx")):
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
        if pack_id == "vite" and target.hostname == "www.typescriptlang.org" and target.path.rstrip("/") == "/tsconfig":
            return target._replace(fragment=target.fragment.split(":~:text=", 1)[0]).geturl()
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
    if tag == "pre":
        # Javadoc diagnostics can wrap a nested example in an outer pre.
        if any(descendant.tag == "pre" for child in children if isinstance(child, Node) for descendant in child.walk()):
            tag = "div" if attrs else ""
        elif not as_text(Node("", children=children), pre=True).strip():
            return Node("span", {"id": anchor}) if "id" in attrs else None
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


def html_from_pandoc(raw, source, pack_id, xrefs, assets=None, *, input_format="gfm+attributes+fenced_divs"):
    result = subprocess.run(
        ("pandoc", f"--from={input_format}", "--to=html5", "--syntax-highlighting=none"),
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


def html_from_markdown(raw, source, pack_id, xrefs, assets=None):
    return html_from_pandoc(raw, source, pack_id, xrefs, assets)


def transform_prose(raw, transform):
    sections = re.split(r"(^[ \t]*(`{3,}|~{3,})[^\n]*\n.*?^[ \t]*\2[`~]*[ \t]*$)",
                        raw, flags=re.M | re.S)
    return "".join(transform(section) if index % 3 == 0 else section
                   for index, section in enumerate(sections) if index % 3 != 2)


def front_matter(raw, key):
    if not raw.startswith("---"):
        return None
    header = raw.split("---", 2)[1]
    field = re.search(rf"^{key}:[ \t]*(.*?)\s*$", header, re.M)
    return field[1].strip("\"'") if field else None
