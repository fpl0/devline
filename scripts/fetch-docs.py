#!/usr/bin/env python3
"""Build DevKit offline docs from pinned upstream artifacts and local toolchains.

See --help for source inputs. This script never changes the input archives,
reference tree, or toolchain installations. It uses only Python's standard library.
"""

import argparse
import ast
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from html.parser import HTMLParser
import html
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile
from urllib.parse import parse_qs, quote, urlsplit
from urllib.request import Request, urlopen
import zipfile


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
}

VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}
OMIT = {"aside", "button", "footer", "form", "head", "header", "nav", "noscript", "script", "style", "svg", "template"}
BLOCK = {"article", "blockquote", "dd", "div", "dl", "dt", "h1", "h2", "h3", "h4", "h5", "h6",
         "hr", "li", "main", "ol", "p", "pre", "section", "table", "tr", "ul"}
PRESENTATION = {"anchor", "headerlink", "visually-hidden", "sr-only", "sidebar", "copy-button",
                "breadcrumb", "breadcrumbs", "pagination", "screen-reader-only", "toc", "source-link"}


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
    text = "".join(as_text(child, pre or node.tag == "pre") for child in node.children)
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
        key = n.attrs.get("id") or ""
        if ((n.tag == "dt" and "." in key)
                or (n.tag in {"h2", "h3", "section", "a"} and re.match(
                    r"^(?:method\.|tymethod\.|associatedconstant\.|fn\.|[a-zA-Z_]\w*[!?]?/\d+$)", key))
                or (mode == "javadoc" and n.tag == "section" and "(" in key)):
            aliases.append(key)
    return title or title_fallback, clean_text(as_text(main)), aliases


def read_text(file):
    return file.read_text(encoding="utf-8-sig", errors="replace")


def markdown_article(raw, fallback):
    raw = re.sub(r"\A\s*---\s*\n.*?\n---[ \t]*\n", "", raw, count=1, flags=re.S)
    raw = re.sub(r"<!--.*?-->", "", raw, flags=re.S)
    raw = re.sub(r"\{ #[-\w]+ \}", "", raw)
    raw = re.sub(r"\A\*\*DO NOT READ THIS FILE ON GITHUB.*?\*\*\s*", "", raw, flags=re.S)
    title = re.search(r"^#\s+([^\n]+)|^([^\n]+)\n=+[ \t]*$", raw, re.M)
    title = (title.group(1) or title.group(2)).strip() if title else fallback
    raw = re.sub(r"!\[([^\]]*)\]\([^)]*\)", r"\1", raw)
    raw = re.sub(r"\[([^\]]+)\]\([^\n]*?\)", r"\1", raw)
    raw = re.sub(r"^:{3,}\s*moniker\s+range=.*$|^:{3,}\s*moniker-end\s*$", "", raw, flags=re.M)
    raw = re.sub(r"^///(?:\s+([\w -]+))?\s*$", lambda m: (m.group(1) or "").title(), raw, flags=re.M)
    raw = re.sub(r"</?(?:div|span|font|u|b)(?:\s[^>\n]*)?>", "", raw)
    return title, clean_text(html.unescape(raw))


def slug_title(file):
    return file.stem.replace("-", " ").replace("_", " ")


MARKDOWN_PATHS = {
    "fastapi": ["*"],
    "csharp": ["*"],
    "aspnet": ["*"],
    "chi": ["README.md"],
    "ruby": ["api/*", "method/*", "*.md"],
    "rails": ["*"],
}


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

    def add(self, key, title, body, source, aliases=()):
        if len(body) < 80:
            return
        body_chunks = []
        # Split large references at paragraph boundaries so the CLI can open a manageable page.
        while len(body) > 45_000:
            cut = body.rfind("\n\n", 31_000, 45_000)
            if cut < 0:
                cut = body.rfind("\n", 31_000, 45_000)
            if cut < 0:
                cut = 45_000
            body_chunks.append(body[:cut])
            body = body[cut:].lstrip()
        body_chunks.append(body)
        for index, chunk in enumerate(body_chunks):
            part_aliases = list(aliases) if index == 0 else []
            self.documents.append({
                "id": key if index == 0 else f"{key}::part-{index + 1}",
                "title": title if index == 0 else f"{title} (part {index + 1})",
                "body": chunk,
                "source_url": source,
                "aliases": list(dict.fromkeys(a for a in part_aliases if a and len(a) < 200))[:350],
            })

    def notice(self, path, name=None):
        self.notices[name or path.name] = read_text(path)

    def emit(self, data):
        if not self.documents:
            raise RuntimeError(f"{self.id} has no documents")
        ids = [document["id"] for document in self.documents]
        if len(ids) != len(set(ids)):
            raise RuntimeError(f"Duplicate document IDs in {self.id}")
        target = data / "docs" / self.id / self.version
        target.mkdir(parents=True, exist_ok=True)
        atomic_json(target / "documents.json", self.documents)
        for name, content in self.notices.items():
            (target / name).write_text(content, encoding="utf-8")
        (target / "SOURCES.md").write_text(
            f"# {self.name} {self.version}\n\nSource: {self.source_url}\n\n"
            f"{self.source_detail}\n\nLicense: {self.license}. See the accompanying notices. "
            "DevKit extracted article text, removed site navigation and markup, and split long pages "
            "for search. This is a reformatted offline snapshot. Each record has its upstream source link.\n",
            encoding="utf-8",
        )
        print(f"{self.id:<13} {len(self.documents):>5} records {(target / 'documents.json').stat().st_size / 2**20:>6.1f} MiB", flush=True)
        metadata = {
            "id": self.id, "name": self.name, "stack": self.stack,
            "version": self.version, "description": self.description,
            "source_url": self.source_url, "license": self.license,
            "path": target.relative_to(data).as_posix(),
        }
        if self.id in MARKDOWN_PATHS:
            metadata["markdown"] = MARKDOWN_PATHS[self.id]
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


def html_page(pack, raw, key, *, mode="main", source=None, title_prefix="", aliases=()):
    title, body, extracted = article_from_html(raw, mode, slug_title(Path(key)))
    if body:
        qualified = []
        if aliases and mode == "hex":
            qualified = [f"{aliases[0]}.{item}" for item in extracted]
        elif aliases and mode == "javadoc":
            qualified = [f"{name}.{item.partition('(')[0]}" for name in aliases for item in extracted]
        elif aliases and (pack.stack == "rust" or pack.id == "axum"):
            qualified = [f"{aliases[0]}::{item.partition('.')[2]}" for item in extracted if "." in item]
        pack.add(key, title_prefix + title, body, source or pack.source_url + quote(key, safe="/.-"),
                 [*aliases, *qualified, *extracted])


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
            html_page(pack, raw, key, mode="sphinx", aliases=(file.stem,))
    return pack


def include_fastapi_examples(raw, root):
    def substitute(match):
        name = match.group(1).split(":")[0].strip()
        example = root / "docs_src" / name
        if not example.is_file():
            return ""
        return "\n```python\n" + read_text(example).rstrip() + "\n```\n"

    return re.sub(r"\{\*\s+(?:\.\./)+docs_src/(.*?)\s+\*\}", substitute, raw)


def fastapi_pack(cache, scratch):
    pack = Pack("fastapi", "FastAPI", "python", "0.162.0",
                "Official FastAPI tutorial, advanced user guide and deployment guides.",
                "https://fastapi.tiangolo.com/", "MIT",
                "FastAPI contributors, repository tag 0.162.0. "
                "https://github.com/fastapi/fastapi/tree/0.162.0/docs/en/docs")
    safe_extract_tar(cache / "fastapi-0.162.0.tar.gz", scratch, lambda name:
                     name.name == "LICENSE" or str(name).partition("/")[2].startswith(("docs/en/docs/", "docs_src/", "fastapi/")))
    root = scratch / "fastapi-0.162.0"
    pack.notice(root / "LICENSE")
    base = root / "docs/en/docs"
    for file in sorted(base.rglob("*.md")):
        key = file.relative_to(base).as_posix()
        if key.startswith(("release-notes", "external-links", "_", "reference/", "img/", "benchmarks")):
            continue
        title, body = markdown_article(include_fastapi_examples(read_text(file), root), slug_title(file))
        if not body.startswith(":::"):
            route = key.removesuffix(".md").removesuffix("index").rstrip("/")
            pack.add(key, title, body, pack.source_url + (route + "/" if route else ""), (file.stem, "FastAPI " + title))
    api = {
        "applications": ("fastapi", {"FastAPI"}),
        "routing": ("apirouter", {"APIRouter"}),
        "exceptions": ("exceptions", {"HTTPException", "WebSocketException"}),
        "param_functions": ("parameters", {"Query", "Path", "Body", "Cookie", "Header", "File", "Form"}),
    }
    for module, (route, symbols) in api.items():
        code = ast.parse(read_text(root / "fastapi" / f"{module}.py"))
        for node in code.body:
            if not isinstance(node, (ast.ClassDef, ast.FunctionDef)) or node.name not in symbols:
                continue
            body = ast.get_docstring(node)
            if body:
                qualified = f"fastapi.{node.name}"
                pack.add("api/" + node.name, node.name, body,
                         f"{pack.source_url}reference/{route}/#{qualified}", (qualified,))
    return pack


def docfx_includes(raw, file, base, depth=0):
    def source_path(target):
        target = target.split("?", 1)[0].split("#", 1)[0].strip()
        path = (base / target[2:] if target.startswith("~/") else file.parent / target).resolve()
        return path if path.is_relative_to(base.resolve()) and path.is_file() else None

    def include(match):
        path = source_path(match.group(1))
        if depth >= 3 or not path or path.suffix != ".md":
            return ""
        included = re.sub(r"\A\s*---\s*\n.*?\n---[ \t]*\n", "", read_text(path), count=1, flags=re.S)
        return docfx_includes(included, path, base, depth + 1)

    seen = set()

    def snippet(language, target, region=""):
        path = source_path(target)
        parts = urlsplit(target)
        region = region or parse_qs(parts.query).get("name", [""])[0] or parts.fragment
        signature = (path, region)
        if signature in seen:
            return ""
        seen.add(signature)
        missing = "See the upstream article for this code sample."
        if not path or path.stat().st_size > 100_000:
            return missing
        content = read_text(path)
        if region:
            region = re.escape(region)
            match = re.search(
                r"(?mi)^\s*(?:#region\s+|//\s*<)(?:snippet)?"
                + region + r">?\s*\n(.*?)(?=^\s*(?:#endregion|//\s*</(?:snippet)?"
                + region + r">))", content, re.S,
            )
            if not match:
                return missing
            content = match.group(1)
        if len(content) > 6_000:
            return missing
        return "\n```" + language + "\n" + content.rstrip() + "\n```\n"

    def code_block(match):
        region = re.search(r'\bid="([^"]+)"', match.group())
        return snippet(*match.groups(), region=region.group(1) if region else "")

    raw = re.sub(r"\[!INCLUDE\s*\[[^\]]*\]\(([^)]+)\)\]", include, raw)
    raw = re.sub(r"\[!code-([\w+-]+)\s*\[[^\]]*\]\(([^)]+)\)\]",
                 lambda match: snippet(*match.groups()), raw)
    return re.sub(r'^\s*:::code[^\n]*?language="([^"]+)"[^\n]*?source="([^"]+)"[^\n]*:::\s*$',
                  code_block, raw, flags=re.M)


def microsoft_pack(reference, *, aspnet=False):
    microsoft = reference / "microsoft"
    if aspnet:
        repo = microsoft / "AspNetCore.Docs"
        pack = Pack("aspnet", "ASP.NET Core", "dotnet", "2026.11.06",
                    "Microsoft Learn ASP.NET Core fundamentals, Web APIs and tutorials snapshot.",
                    "https://learn.microsoft.com/en-us/aspnet/core/", "CC-BY-4.0; accompanying code MIT",
                    read_text(repo / "SNAPSHOT.txt"))
        base = repo / "aspnetcore"
    else:
        repo = microsoft / "docs"
        pack = Pack("csharp", "C# and .NET", "dotnet", "2026.11.06",
                    "Microsoft Learn C# language, runtime, tooling and .NET fundamentals snapshot.",
                    "https://learn.microsoft.com/en-us/dotnet/", "CC-BY-4.0; accompanying code MIT",
                    read_text(repo / "SNAPSHOT.txt"))
        base = repo / "docs"
    pack.notice(repo / "LICENSE")
    pack.notice(repo / "LICENSE-CODE")
    for file in sorted(base.rglob("*.md")):
        key = file.relative_to(base).as_posix()
        if any(part in {"includes", "_includes", "snippets", "media"} for part in file.relative_to(base).parts):
            continue
        raw = read_text(file)
        if re.search(r"^redirect_url:", raw, re.M):
            continue
        title, body = markdown_article(docfx_includes(raw, file, base), slug_title(file))
        path = key.removesuffix(".md")
        pack.add(key, title, body, pack.source_url + path, (file.stem, "ASP.NET " + title if aspnet else "C# " + title))
    return pack


GO_PACKAGES = """archive/tar archive/zip bufio bytes cmp compress/gzip context crypto/aes
crypto/hmac crypto/rand crypto/sha256 crypto/tls database/sql embed encoding/base64
encoding/csv encoding/gob encoding/hex encoding/json encoding/xml errors flag fmt
hash html/template io io/fs iter log log/slog maps math math/rand/v2 mime/multipart
net net/http net/http/httptest net/netip net/url os os/exec os/signal path path/filepath
reflect regexp runtime slices sort strconv strings sync sync/atomic testing text/template
time unicode unicode/utf8""".split()


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
        pack.add("std/" + package, package, clean_text(body), f"https://pkg.go.dev/{package}@go{version}", aliases)

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
        chi.add("api" + tail, "Chi " + (tail.removeprefix("/") or "Router"), clean_text(body),
                f"https://pkg.go.dev/{mod}@v{version}{tail}", aliases)
    for file in [chi_root / "README.md", *sorted((chi_root / "_examples").glob("*/main.go"))]:
        if not file.is_file():
            continue
        rel = file.relative_to(chi_root).as_posix()
        if file.suffix == ".go":
            title, body = "Chi example: " + file.parent.name.replace("-", " "), read_text(file)
        else:
            title, body = markdown_article(read_text(file), "Chi README")
        chi.add(rel, title, body, f"https://github.com/go-chi/chi/blob/v{version}/{rel}", (file.parent.name,))
    return pack, chi


def java_pack(java_home, scratch):
    version = "26.0.2.1"
    if f'JAVA_VERSION="{version}"' not in read_text(java_home / "release").splitlines():
        raise RuntimeError(f"Java pack requires JDK {version}: {java_home}")
    pack = Pack("java", "Java SE", "java", version,
                "Official OpenJDK API reference generated from curated JDK 26 library sources.",
                "https://docs.oracle.com/en/java/javase/26/docs/api/", "GPL-2.0 with Classpath Exception",
                "OpenJDK contributors, Javadoc generated from the Liberica JDK 26.0.2.1 src.zip. "
                "The upstream links point to the corresponding Java SE reference pages.")
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
            html_page(pack, read_text(file), rel, mode="javadoc", aliases=aliases)
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
        html_page(pack, read_text(file), rel.as_posix(), mode="spring", aliases=(file.stem,), title_prefix="Spring Boot · ")
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
    for file in sorted((root / "doc").rglob("*")):
        if file.suffix not in {".rdoc", ".md"} or len(file.relative_to(root / "doc").parts) > 2:
            continue
        if file.stem.startswith(("NEWS", "ChangeLog", "contributing", "maintainers")):
            continue
        key = file.relative_to(root / "doc").as_posix()
        title, body = markdown_article(read_text(file), slug_title(file))
        title = re.sub(r"^=+\s+", "", title)
        suffix = "_" + file.suffix.removeprefix(".") + ".html"
        route = "doc/" + key.removesuffix(file.suffix) + suffix
        pack.add("doc/" + key, title, body, pack.source_url + route, (file.stem,))
    ri = ruby.with_name("ri")
    for cls in RUBY_CLASSES:
        body = command(ri, "--no-pager", "--format=markdown", "--no-gems", cls, check=False)
        if body:
            url = pack.source_url + cls.replace("::", "/") + ".html"
            pack.add("api/" + cls, cls, clean_text(body), url, (cls,))
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
            pack.add("method/" + symbol, symbol, clean_text(body),
                     pack.source_url + prefix.replace("::", "/") + ".html" + anchor, (symbol,))
    return pack


def rails_pack(cache, scratch, rust_docs):
    pack = Pack("rails", "Ruby on Rails", "ruby", "8.1.3",
                "Official Rails guides for routing, controllers, views, models, jobs and deployment.",
                "https://guides.rubyonrails.org/v8.1/", "CC-BY-SA-4.0",
                "Ruby on Rails contributors, guides/source at tag v8.1.3. "
                "Adapted as searchable text under the same CC BY-SA 4.0 license.")
    pack.notice(rust_docs.parent / "licenses/CC-BY-SA-4.0.txt", "LICENSE")
    safe_extract_tar(cache / "rails.tar.gz", scratch,
                     lambda name: str(name).partition("/")[2].startswith("guides/source/") and name.suffix == ".md")
    base = scratch / "rails-8.1.3/guides/source"
    for file in sorted(base.glob("*.md")):
        if re.match(r"^[0-9]|^upgrading", file.name):
            continue
        title, body = markdown_article(read_text(file), slug_title(file))
        pack.add(file.name, title, body, pack.source_url + file.stem + ".html", (file.stem, "Rails " + title))
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
            html_page(pack, read_text(file), rel, aliases=(file.stem,))
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
        html_page(pack, read_text(file), "std/" + relative.as_posix(), aliases=aliases)
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
        html_page(pack, read_text(file), key, aliases=aliases, title_prefix="Axum · ")
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
        safe_extract_tar(cache / "elixir-docs.tar.gz", base, lambda file: file.suffix == ".html")
    else:
        base = reference / "hexdocs" / name
    for file in sorted(base.glob("*.html")):
        if file.stem in {"404", "api-reference", "search", "readme", "changelog"}:
            continue
        html_page(pack, read_text(file), file.name, mode="hex", aliases=(file.stem,))
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
            "fastapi": (("fastapi-0.162.0.tar.gz",), lambda: fastapi_pack(cache, scratch)),
            "csharp": ((), lambda: microsoft_pack(reference)),
            "aspnet": ((), lambda: microsoft_pack(reference, aspnet=True)),
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
