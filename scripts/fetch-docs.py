#!/usr/bin/env python3
"""Build Devline offline docs from pinned upstream artifacts and local toolchains.

See --help for source inputs. This script never changes the input archives,
reference tree, or toolchain installations. Pandoc 3.13 renders Markdown at build
time. The resulting snapshots have no runtime rendering dependency.
"""

import argparse
import ast
import copy
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from filecmp import cmp
import html
import json
import os
from pathlib import Path
import re
import shutil
import tarfile
import tempfile
import textwrap
from urllib.parse import parse_qs, quote, unquote, urljoin, urlsplit
from urllib.request import Request, urlopen
import zipfile

from infrastructure_sources import REVISION as INFRASTRUCTURE_REVISION, UPSTREAM as INFRASTRUCTURE_UPSTREAM
from infrastructure_sources import VERSION as INFRASTRUCTURE_VERSION, document_id, read_sources, source_path
from reference_io import atomic_json, command, read_text, safe_extract_tar, write_shards
from reference_rendering import (
    IMAGE_SUFFIXES, PANDOC_VERSION, Node, archive_assets, article_from_html, as_text, clean_text,
    code_fence, file_assets, front_matter, html_from_markdown, html_from_pandoc, markdown_article,
    prepared_html, render_html, transform_prose, without_front_matter,
)


DOTNET_COMMIT = "37fe0bfc5f596bf0bafd74bf927010384f184d96"
ASPNET_COMMIT = "47d6f5c46f7a4c552fbf8ca9077439b95150c719"
ASPNET_SAMPLES_COMMIT = "8dbb6f86c84f78cfd4a61f9207d55199635595bf"
ASPNET_SAMPLES_ARCHIVE = f"aspnet-samples-{ASPNET_SAMPLES_COMMIT}.tar.gz"
NODE_VERSION = "26.13.0"
NODE_DOC_BASE = f"https://nodejs.org/dist/v{NODE_VERSION}/docs/api/"
NODE_PAGES = (
    "assert async_context buffer child_process cli console crypto dgram dns events fs http http2 https "
    "module net os path perf_hooks process readline sqlite stream test timers tls url util worker_threads zlib esm packages typescript"
).split()
REACT_DOC_COMMIT = "2e6dca162985d438d7a834b339d89177cf0a92b9"
TYPESCRIPT_DOC_COMMIT = "a3ed0be5249fd75fae9c2f527de23f0fc16c5d0d"
WEB_ARCHIVES = {
    "playwright-v1.65.0.tar.gz": "https://codeload.github.com/microsoft/playwright/tar.gz/refs/tags/v1.65.0",
    "vite-v8.3.6.tar.gz": "https://codeload.github.com/vitejs/vite/tar.gz/refs/tags/v8.3.6",
    "react-docs-2e6dca1.tar.gz": f"https://codeload.github.com/reactjs/react.dev/tar.gz/{REACT_DOC_COMMIT}",
    "typescript-docs-a3ed0be.tar.gz": f"https://codeload.github.com/microsoft/TypeScript-Website/tar.gz/{TYPESCRIPT_DOC_COMMIT}",
}
NODE_DOWNLOADS = {
    f"node{NODE_VERSION}-{page}.html": NODE_DOC_BASE + f"{page}.html"
    for page in NODE_PAGES
}
NODE_DOWNLOADS[f"node{NODE_VERSION}-LICENSE"] = f"https://raw.githubusercontent.com/nodejs/node/v{NODE_VERSION}/LICENSE"
NODE_ANCHOR_REPAIRS = {
    "cli.html": {"-p---print-script": "-print-script", "-r---require-module": "-require-module",
                 "--force-fips": "--force-fipsmode", "--experimental-test-tag-filtertag": "--experimental-test-tag-filterexpr"},
    "fs.html": {"fsfswatcher": "class-fsfswatcher"},
    "readline.html": {"class-readlineinterfaceconstructor": "class-interfaceconstructor"},
    "tls.html": {"tlstlssocket": "class-tlstlssocket", "class-tlssecurecontext": "tlscreatesecurecontextoptions"},
    "net.html": {"event-error_1": "event-error-1"},
    "process.html": {"processexitcode_1": "processexitcode-1"},
    "worker_threads.html": {"event-message_1": "event-message-1"},
}

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
    **WEB_ARCHIVES,
    **NODE_DOWNLOADS,
}

CONTENT_REVISION = "devline.1"
CONTENT_REVISIONS = {
    "csharp": "devline.5", "aspnet": "devline.4", "python": "devline.2",
    "go": "devline.3", "java": "devline.2", "rails": "devline.2", "ruby": "devline.2",
    "infrastructure": INFRASTRUCTURE_REVISION,
}
REFRESHED_SNAPSHOTS = {
    ("csharp", "2026.11.06+devline.5"), ("aspnet", "2026.11.06+devline.4"),
    ("go", "1.27.3+devline.3"), ("java", "26.0.2.1+devline.2"),
    ("chi", "5.5.0+devline.1"),
    ("infrastructure", f"{INFRASTRUCTURE_VERSION}+{INFRASTRUCTURE_REVISION}"),
    ("node", "26.13.0+devline.1"), ("typescript", "7.0.2+devline.1"),
    ("playwright", "1.65.0+devline.1"), ("react", "19.3.1+devline.1"), ("vite", "8.3.6+devline.1"),
}
PACK_SCOPE = {
    "python": ("Tutorial, language reference, HOWTOs, and standard library API pages", "Packaging tools and third-party libraries are separate references"),
    "fastapi": ("Tutorial, advanced patterns, deployment, and generated API pages", "Translated guides and upstream release history are omitted"),
    "csharp": ("C# language, .NET fundamentals, SDK tooling, and compatibility notes", "The full .NET API catalog is outside this snapshot"),
    "aspnet": ("ASP.NET Core fundamentals, Web APIs, security, tests, hosting, and tutorials", "The full ASP.NET API catalog and version variants are outside this snapshot"),
    "go": ("Language specification, memory model, assembler, and selected standard library packages", "Packages outside the curated standard library selection and third-party modules are omitted"),
    "chi": ("Router and middleware APIs, README, and examples", "Third-party middleware modules are omitted"),
    "java": ("OpenJDK API pages for selected core modules and classes", "JDK packages outside the selected modules and platform guides are omitted"),
    "spring-boot": ("Spring Boot reference, how-to guides, and tutorials", "Spring Framework and external Spring projects are separate references"),
    "ruby": ("Language guides, selected standard library classes, and methods from RDoc", "Gem documentation and unselected RDoc entries are omitted"),
    "rails": ("Application guides, Active Record, controllers, routing, jobs, and deployment", "The full Rails API class catalog is omitted"),
    "rust": ("Rust Book, Reference, Cargo Book, and selected standard library APIs", "Nightly-only and unselected Rustdoc pages are omitted"),
    "axum": ("Rustdoc for routing, extractors, responses, and middleware", "Transitive crate APIs are separate references"),
    "elixir": ("Language guides and core and standard library APIs", "Erlang/OTP documentation is separate"),
    "phoenix": ("Framework guides and Phoenix API modules", "External Phoenix packages have separate documentation"),
    "ecto": ("Ecto guides and database library API modules", "External adapters and Ecto SQL documentation are separate"),
    "infrastructure": ("Official documentation for Devline's infrastructure catalog",
                       "See the service-specific coverage and omissions in SOURCES.md"),
    "node": ("Official Node.js APIs for file I/O, HTTP, streams, SQLite, tests, modules, workers, and related core libraries",
             "Unselected Node APIs, release history, and npm packages are outside this snapshot"),
    "typescript": ("The official TypeScript Handbook and selected TSConfig compiler options",
                   "Interactive compiler playgrounds, localized pages, and the full compiler API are omitted"),
    "playwright": ("JavaScript and TypeScript browser testing guides, API requests, fixtures, configuration, and selected API classes",
                   "Other language examples and unselected API classes are omitted"),
    "react": ("Official React fundamentals and selected component and Hook references",
              "Interactive sandboxes and unselected learning and API pages are omitted"),
    "vite": ("Official Vite guides for setup, development, building, assets, and configuration",
             "Framework plugin manuals and interactive media are outside this snapshot"),
}

SNAPSHOT_OMISSIONS = {
    ("aspnet", "2026.11.06+devline.3"): [
        "59 links target headings absent from this snapshot",
        "One source example is blank",
    ],
    ("aspnet", "2026.11.06+devline.4"): ["59 links target headings absent from this snapshot"],
    ("csharp", "2026.11.06+devline.4"): [
        "One link targets a heading absent from this snapshot",
        "One source example is blank",
    ],
    ("csharp", "2026.11.06+devline.5"): ["One link targets a heading absent from this snapshot"],
    ("elixir", "1.20.5+devline.1"): ["Two links target headings absent from this snapshot"],
    ("java", "26.0.2.1+devline.1"): [
        "One link targets a heading absent from this snapshot",
        "One Javadoc example contains nested code markup",
    ],
    ("java", "26.0.2.1+devline.2"): ["One link targets a heading absent from this snapshot"],
    ("python", "3.15.2+devline.2"): ["Two links target headings absent from this snapshot"],
    ("rails", "8.1.3+devline.2"): ["Eight links target headings absent from this snapshot"],
    ("ruby", "4.0.9+devline.2"): ["One link targets a heading absent from this snapshot"],
    ("rust", "1.99.1+devline.1"): ["Six links target headings absent from this snapshot"],
    ("spring-boot", "4.1.2+devline.1"): ["Five links target headings absent from this snapshot"],
}


def pack_metadata(pack_id, upstream, snapshot_version, missing=0):
    coverage, omission = PACK_SCOPE[pack_id]
    expanded_go = (pack_id, snapshot_version) == ("go", "1.27.3+devline.3")
    result = {
        "upstream_version": "10.0" if pack_id in {"csharp", "aspnet"} else upstream,
        "snapshot_date": "2026-11-07" if (pack_id, snapshot_version) in REFRESHED_SNAPSHOTS else "2026-11-06",
        "coverage": [coverage],
        "omissions": [omission, *SNAPSHOT_OMISSIONS.get((pack_id, snapshot_version), [])],
    }
    if expanded_go:
        result["coverage"].append("Expanded networking, cryptography, image formats, and filesystem/concurrency testing APIs")
    if missing:
        result["omissions"].append(f"{missing} unavailable source includes or samples are listed in MISSING-INCLUDES.txt")
    return result


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


def slug_title(file):
    return file.stem.replace("-", " ").replace("_", " ")


def document_category(pack, key):
    area, _, rest = key.partition("/")
    if pack == "infrastructure":
        return "reference" if area == "reference" else "guide"
    if pack == "node" or pack == "playwright" and area == "api" or pack == "react" and area == "reference":
        return "api"
    if pack in {"typescript", "playwright", "react", "vite"}:
        return "reference" if area in {"tsconfig", "config"} else "guide"
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
    scope: dict = field(default_factory=dict)

    def add(self, key, title, body, source, aliases=(), markup=None):
        if not body or (self.id != "infrastructure" and len(body) < 40):
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
        metadata.update(pack_metadata(self.id, self.version, snapshot_version, len(self.missing)))
        metadata.update(self.scope)
        if shard_count > 1:
            metadata["shards"] = shard_count
        return metadata


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
time unicode unicode/utf8
compress/bzip2 compress/flate compress/lzw compress/zlib crypto/ecdsa crypto/ed25519
crypto/rsa crypto/subtle crypto/x509 encoding/ascii85 encoding/binary encoding/pem html
image image/color image/draw image/gif image/jpeg image/png mime net/http/cookiejar
net/http/httptrace net/http/httputil net/mail net/smtp net/textproto os/user runtime/metrics
testing/fstest testing/iotest testing/quick testing/synctest text/scanner unicode/utf16""".split()


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


def infrastructure_pack(upstream, catalog, scratch):
    services = json.loads(read_text(catalog))["services"]
    sources = list(read_sources(services, upstream))
    license_names = dict.fromkeys(source["license"] for _, _, source in sources if source["documents"])
    pack = Pack(
        "infrastructure", "Infrastructure References", "infrastructure", INFRASTRUCTURE_VERSION,
        "Official publisher documentation for Devline's local services. See coverage for each service.",
        "https://github.com/fpl0/devline/tree/main/scripts/infrastructure-docs/upstream",
        "; ".join(license_names),
        f"Source snapshot date: {INFRASTRUCTURE_VERSION.replace('.', '-')}. "
        "Devline's setup instructions are published separately as Devline Help.",
    )
    pack.scope = {"coverage": [], "omissions": []}
    for service, root, source in sources:
        name, service_id = service["name"], service["id"]
        scope = f"{name} {service['version']}"
        notices = []
        for notice in source["notices"]:
            label = f"{service_id.upper()}-{Path(notice).name}"
            pack.notice(source_path(root, notice), label)
            notices.append(f"`{label}`")
        pack.scope["coverage"].append(f"{scope} — {source['coverage']}")
        pack.scope["omissions"].append(f"{scope} — {source['omissions']}")
        pack.source_detail += (
            f"\n\n## {scope}\n\n{source['publisher']}. {source['provenance']}\n\n"
            f"Coverage: {source['coverage']}\n\nOmissions: {source['omissions']}\n\n"
            f"License: {source['license']}"
            + (f". See {', '.join(notices)}." if notices else ".")
        )
        if archive := source.get("html_archive"):
            archive_root = scratch / service_id
            prefix = Path(archive["root"])
            safe_extract_tar(source_path(root, archive["path"]), archive_root,
                             lambda path: path.is_relative_to(prefix))
            root = archive_root
        for page in source["documents"]:
            add_infrastructure_page(pack, service, root, page)
    return pack


def add_infrastructure_page(pack, service, root, page):
    file = source_path(root, page["path"])
    url = page["source_url"]
    key = document_id(service["id"], page["id"])
    assets = file_assets(file, root)
    markdown = file.suffix in {".md", ".mdx"}
    raw = read_text(file)
    aliases = page.get("aliases", [])
    if markdown:
        title, raw = markdown_article(raw, slug_title(file))
    elif file.suffix == ".html":
        title, article, html_aliases = article_from_html(raw, "main", slug_title(file))
        aliases = [*aliases, *html_aliases]
        if selector := page.get("selector"):
            article = next((node for node in article.walk() if
                            node.tag == selector.get("tag", node.tag)
                            and node.attrs.get("id") == selector.get("id", node.attrs.get("id"))
                            and ("class" not in selector or selector["class"] in node.attrs.get("class", "").split())), None)
            if article is None:
                raise RuntimeError(f"Publisher article selector did not match: {url}")
        article.children = [child for child in article.children if not isinstance(child, Node)
                            or child.attrs.get("class") not in {"navheader", "navfooter"}]
        if service["id"] == "postgres":
            for node in article.walk():
                if node.tag == "object" and node.attrs.get("type") == "image/svg+xml":
                    node.tag = "img"
                    node.attrs = {"src": node.attrs["data"], "alt": clean_text(as_text(node))}

            def postgres_assets(path):
                content = assets(path)
                if not content:
                    return None
                # Drop the generated DocBook DTD declaration without resolving it.
                return re.sub(br'<!DOCTYPE svg PUBLIC "[^"]+" "[^"]+">', b"", content, count=1)

            body, markup = prepared_html(article, url, pack.id, assets=postgres_assets)
        else:
            body, markup = prepared_html(article, url, pack.id, assets=assets)
    elif re.fullmatch(r"\.[1-9]", file.suffix):
        title = file.stem
        body, markup = html_from_pandoc(raw, url, pack.id, pack.xrefs, assets, input_format="man")
    else:
        raise RuntimeError(f"Unsupported infrastructure reference format: {file}")
    title = page.get("title") or title
    name = service["name"]
    qualified = title if title.casefold().startswith(name.casefold()) else f"{name} · {title}"
    aliases = [*aliases, title, f"{service['id']} {title}"]
    if markdown:
        pack.add_markdown(key, qualified, raw, url, aliases, assets)
    else:
        pack.add(key, qualified, body, url, aliases, markup)


def node_pack(cache):
    pack = Pack(
        "node", "Node.js", "node", NODE_VERSION, "Official Node.js core API reference.",
        NODE_DOC_BASE, "MIT; bundled upstream notices",
        f"The Node.js contributors, rendered Node.js {NODE_VERSION} API documentation from {NODE_DOC_BASE}.",
    )
    pack.notice(cache / f"node{NODE_VERSION}-LICENSE", "LICENSE")
    for page in NODE_PAGES:
        key = f"{page}.html"
        raw = read_text(cache / f"node{NODE_VERSION}-{key}")
        if f"Node.js v{NODE_VERSION} Documentation" not in raw:
            raise RuntimeError(f"Node API page {key} does not match {NODE_VERSION}")
        title, article, _ = article_from_html(raw, "sphinx", page)
        if article.attrs.get("id") != "apicontent":
            raise RuntimeError(f"Cannot identify Node API content for {key}")
        title = title.partition(" | Node.js")[0]
        aliases = [page, f"node:{page}"]
        for node in list(article.walk()):
            node.children = [child for child in node.children
                             if not isinstance(child, Node) or child.attrs.get("id") not in {"toc", "toc-toggle"}]
            if "mark" in (node.attrs.get("class") or "").split():
                node.tag = "span"
                node.attrs = {"id": node.attrs["id"]} if "id" in node.attrs else {}
                node.children = []
            if node.tag == "a":
                href = node.attrs.get("href") or ""
                target = urlsplit(href)
                if target.path.startswith("/api/") and target.hostname in {None, "nodejs.org"}:
                    target = urlsplit(urljoin(NODE_DOC_BASE, target.path.removeprefix("/api/"))) \
                        ._replace(fragment=target.fragment)
                if target.hostname in {None, "nodejs.org"}:
                    page_name = Path(target.path).name or key
                    corrected = NODE_ANCHOR_REPAIRS.get(page_name, {}).get(target.fragment, target.fragment)
                    node.attrs["href"] = target._replace(fragment=corrected).geturl()
            if node.tag in {"h2", "h3", "h4", "h5"}:
                for child in node.walk():
                    if child.tag != "code":
                        continue
                    symbol = re.match(r"[A-Za-z_$][\w$.:/]*", as_text(child).strip())
                    if symbol:
                        name = symbol[0]
                        aliases.extend((name, name.replace("fsPromises.", "fs.promises.")))
        # Article links retain the upstream IDs and version-specific release URLs.
        body, markup = prepared_html(article, NODE_DOC_BASE + key, pack.id)
        pack.add(key, title, body, NODE_DOC_BASE + key, aliases, markup)
    return pack


def tar_texts(cache, name, accept):
    result = {}
    with tarfile.open(cache / name) as archive:
        for member in archive:
            key = member.name.partition("/")[2]
            if member.isfile() and accept(key):
                result[key] = archive.extractfile(member).read().decode("utf-8")
    return result


def prose_markdown(raw, pack):
    raw = without_front_matter(raw)

    def replace(prose):
        if pack == "react":
            prose = re.sub(r'^<Recipes titleText="([^"]+)" titleId="([\w.-]+)">[ \t]*$', r"### \1 {#\2}\n", prose, flags=re.M)
        prose = re.sub(r"^[ \t]*</?[A-Z][A-Za-z0-9]*(?:\s+[^<>]*?)?\s*/?>[ \t]*\n", "", prose, flags=re.M)
        if pack == "react":
            prose = re.sub(r"[ \t]*\{\s*/\*\s*([\w.-]+)\s*\*/\s*\}[ \t]*$", r" {#\1}", prose, flags=re.M)
        if pack in {"playwright", "vite"}:
            prose = re.sub(r"^:::[ \t]*(?:code-group|details[^\n]*)[ \t]*$", "", prose, flags=re.M)
            prose = re.sub(r"^:::[ \t]*([\w-]+)[ \t]*([^\n]*)$", r"**\1 \2**\n", prose, flags=re.M)
            prose = re.sub(r"^:::[ \t]*$", "", prose, flags=re.M)
        if pack == "vite":
            prose = re.sub(r"[ \t]+<NonInheritBadge\s*/>", "", prose)
        return prose

    return transform_prose(raw, replace)


PLAYWRIGHT_GUIDES = {
    "intro-js", "writing-tests-js", "running-tests-js", "test-configuration-js", "api-testing-js", "test-fixtures-js",
    "test-assertions-js", "locators", "network", "test-cli-js", "trace-viewer-intro-js", "trace-viewer",
    "test-projects-js", "test-webserver-js", "test-timeouts-js", "mock", "pages", "authentication-js", "browsers",
}
PLAYWRIGHT_CLASSES = {
    "apirequest", "apirequestcontext", "apiresponse", "browser", "browsercontext", "locator", "locatorassertions",
    "page", "pageassertions", "test", "testconfig", "testinfo", "testoptions", "browsertype", "route",
}


def playwright_slug(symbol):
    separated = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1-\2", symbol)
    separated = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "-", separated)
    return re.sub(r"[.#]", "-", separated).lower()


def playwright_parameters(raw):
    sections = re.split(r"^## ([\w-]+)[ \t]*$", raw, flags=re.M)
    params = {}
    for key, body in zip(sections[1::2], sections[2::2]):
        langs = re.search(r"^\* langs:[ \t]*(.+)$", body, re.M)
        params[key] = (not langs or "js" in langs[1].replace(",", " ").split(),
                       re.sub(r"^\* langs:[^\n]*\n", "", body, flags=re.M).strip())
    return params


def playwright_expansion(prose, params, routes):
    def option(match):
        heading, token = match.groups()
        javascript, body = params[token]
        return f"{heading}\n\n{body}" if javascript else ""

    prose = re.sub(r"^(#{2,5}[^\n=]+?)\s*=\s*%%-([\w-]+)-%%[ \t]*$", option, prose, flags=re.M)
    prose = re.sub(r"%%-([\w-]+)-%%", lambda match: params[match[1]][1], prose)
    prose = re.sub(r"^\* langs:[^\n]*\n", "", prose, flags=re.M)

    def reference(match):
        name = match[1]
        owner = name.split(".")[0]
        route = routes.get(owner, f"api/class-{owner.lower()}")
        target = route + ("#" + playwright_slug(name) if "." in name else "")
        return f"[`{name}`](https://playwright.dev/docs/{target})"

    return re.sub(r"\[`?(?:method|property|event):\s*([\w.#]+)`?\](?!\()", reference, prose)


def playwright_pack(cache):
    base = "https://playwright.dev/docs/"
    pack = Pack(
        "playwright", "Playwright", "node", "1.65.0", "Official Playwright testing guides and API references.",
        base, "Apache-2.0", "Microsoft and Playwright contributors, from the v1.65.0 documentation source tree. "
        "https://github.com/microsoft/playwright/tree/v1.65.0/docs/src",
    )
    keys = {f"docs/src/{page}.md" for page in PLAYWRIGHT_GUIDES}
    keys.update(f"docs/src/api/class-{page}.md" for page in PLAYWRIGHT_CLASSES)
    params_key = "docs/src/api/params.md"
    sources = tar_texts(cache, "playwright-v1.65.0.tar.gz", lambda key: key in keys or key in {"LICENSE", "NOTICE", params_key})
    pack.notices = {key: sources.pop(key) for key in ("LICENSE", "NOTICE") if key in sources}
    params = playwright_parameters(sources.pop(params_key))
    routes = {name[1]: path.removeprefix("docs/src/").removesuffix(".md") for path, raw in sources.items()
              if (name := re.search(r"^# class:\s+(\w+)", raw, re.M))}
    for path, raw in sorted(sources.items()):
        key = path.removeprefix("docs/src/")
        api = key.startswith("api/")
        slug = front_matter(raw, "id") or key.removesuffix(".md").removesuffix("-js")
        title = front_matter(raw, "title") or key.removesuffix(".md").replace("-", " ")
        aliases = [slug.rsplit("/", 1)[-1]]
        if api:
            symbol = re.search(r"^# class:\s+(\w+)", raw, re.M)
            if symbol:
                title = symbol[1]
                aliases.append(title)
        raw = prose_markdown(raw, pack.id)
        raw = re.sub(r"^# class:[^\n]*\n", "", raw, count=1, flags=re.M)
        raw = re.sub(r"^\x60{3,}(?:csharp|java|python|py|c#)[^\n]*\n.*?^\x60{3,}[ \t]*$", "", raw, flags=re.M | re.S)
        raw = transform_prose(raw, lambda prose: playwright_expansion(prose, params, routes))
        seen = set()

        def headings(prose):
            def heading(match):
                level, kind, name = match.groups()
                anchor = playwright_slug(name)
                if kind == "option" and "." in name:
                    method, parameter = name.rsplit(".", 1)
                    anchor = f"{playwright_slug(method)}-option-{playwright_slug(parameter)}"
                if kind in {"method", "property", "event"}:
                    aliases.append(name.partition("#")[0])
                original = anchor
                number = 2
                while anchor in seen:
                    anchor = f"{original}-{number}"
                    number += 1
                seen.add(anchor)
                return f"{level} `{name.partition('#')[0]}` {{#{anchor}}}"

            prose = re.sub(r"^(#{2,5})\s+(?:async\s+)?(method|property|event|param|option):\s+([\w.#]+)[ \t]*$",
                           heading, prose, flags=re.M)
            return re.sub(r"^\* since:\s*(.+)$", r"**Since** \1", prose, flags=re.M)

        raw = transform_prose(raw, headings)
        pack.add_markdown(key, title, f"# {title}\n\n{raw}", base + slug, aliases)
    return pack


REACT_LEARN = {
    "index", "installation", "creating-a-react-app", "build-a-react-app-from-scratch", "your-first-component",
    "writing-markup-with-jsx", "javascript-in-jsx-with-curly-braces", "passing-props-to-a-component",
    "conditional-rendering", "rendering-lists", "responding-to-events", "state-a-components-memory",
    "state-as-a-snapshot", "updating-objects-in-state", "updating-arrays-in-state", "sharing-state-between-components",
    "synchronizing-with-effects", "you-might-not-need-an-effect", "referencing-values-with-refs", "reusing-logic-with-custom-hooks",
}
REACT_API = {
    "useState", "useEffect", "useRef", "useMemo", "useContext", "useReducer", "useCallback", "useId",
    "useTransition", "useSyncExternalStore", "StrictMode", "Suspense", "memo", "lazy",
}


def react_pack(cache):
    base = "https://react.dev/"
    pack = Pack(
        "react", "React", "node", "19.3.1", "Official React learning guides and selected Hook references.",
        base, "CC-BY-4.0", f"React documentation contributors, React 19 site snapshot at {REACT_DOC_COMMIT}. "
        f"https://github.com/reactjs/react.dev/tree/{REACT_DOC_COMMIT}/src/content",
    )
    keys = {f"src/content/learn/{name}.md" for name in REACT_LEARN}
    keys.update(f"src/content/reference/react/{name}.md" for name in REACT_API)
    sources = tar_texts(cache, "react-docs-2e6dca1.tar.gz", lambda key: key in keys or key == "LICENSE-DOCS.md")
    pack.notices["LICENSE"] = sources.pop("LICENSE-DOCS.md")
    for path, raw in sorted(sources.items()):
        key = path.removeprefix("src/content/").removesuffix(".md")
        title = front_matter(raw, "title") or key.rsplit("/", 1)[-1]
        aliases = (key.rsplit("/", 1)[-1], f"React.{title}")
        pack.add_markdown(key, title, f"# {title}\n\n{prose_markdown(raw, pack.id)}",
                          base + key.removesuffix("/index"), aliases)
    return pack


TS_OPTIONS = (
    "strict", "target", "module", "moduleResolution", "esModuleInterop", "skipLibCheck", "noEmit",
    "isolatedModules", "jsx", "types", "paths", "sourceMap", "declaration", "verbatimModuleSyntax",
    "noEmitOnError", "noImplicitAny", "strictNullChecks", "strictPropertyInitialization", "useDefineForClassFields",
    "downlevelIteration", "baseUrl", "outFile", "rootDirs", "lib", "allowSyntheticDefaultImports",
    "emitDeclarationOnly", "importHelpers", "outDir", "typeRoots",
    "allowArbitraryExtensions", "emitDecoratorMetadata", "exclude", "experimentalDecorators", "extends", "files",
    "importsNotUsedAsValues", "include", "jsxFactory", "jsxFragmentFactory", "jsxImportSource", "noEmitHelpers",
    "preserveValueImports", "references", "rootDir", "allowJs", "composite", "disableFilenameBasedTypeAcquisition",
)


def typescript_pack(cache):
    base = "https://www.typescriptlang.org/"
    pack = Pack(
        "typescript", "TypeScript", "node", "7.0.2", "Official TypeScript Handbook and selected TSConfig options.",
        base, "CC-BY-4.0", f"Microsoft and TypeScript Website contributors, documentation snapshot at {TYPESCRIPT_DOC_COMMIT}. "
        f"https://github.com/microsoft/TypeScript-Website/tree/{TYPESCRIPT_DOC_COMMIT}/packages",
    )
    handbook = "packages/documentation/copy/en/handbook-v2/"
    options = "packages/tsconfig-reference/copy/en/options/"
    sources = tar_texts(cache, "typescript-docs-a3ed0be.tar.gz", lambda key: key == "LICENSE"
                        or key.startswith(handbook) and key.endswith(".md")
                        or key.startswith(options) and Path(key).stem in TS_OPTIONS)
    pack.notices["LICENSE"] = sources.pop("LICENSE")
    compiler_options = []
    for path, raw in sorted(sources.items()):
        name = Path(path).stem
        title = front_matter(raw, "title") or name
        if path.startswith(options):
            compiler_options.append(f"## `{name}` {{#{name}}}\n\n{prose_markdown(raw, pack.id)}")
            continue
        permalink = front_matter(raw, "permalink")
        if not permalink:
            continue
        key = permalink.lstrip("/")
        pack.add_markdown(key, title, f"# {title}\n\n{prose_markdown(raw, pack.id)}", base + key, (name, title))
    pack.add_markdown("tsconfig/options", "TSConfig compiler options",
                      "# TSConfig compiler options\n\n" + "\n\n".join(compiler_options),
                      base + "tsconfig/", TS_OPTIONS)
    return pack


def vite_pack(cache):
    base = "https://v8.vite.dev/"
    pack = Pack(
        "vite", "Vite", "node", "8.3.6", "Official Vite development, build, and configuration guides.",
        base, "MIT", "VoidZero and Vite contributors, from the v8.3.6 documentation source tree. "
        "https://github.com/vitejs/vite/tree/v8.3.6/docs",
    )
    keys = {
        "guide/index", "guide/features", "guide/env-and-mode", "guide/build", "guide/static-deploy", "guide/assets",
        "guide/api-javascript", "guide/backend-integration", "guide/troubleshooting", "guide/dep-pre-bundling",
        "config/index", "config/shared-options", "config/server-options", "config/build-options", "config/preview-options",
    }
    workflow = "docs/guide/static-deploy-github-pages.yaml"
    sources = tar_texts(cache, "vite-v8.3.6.tar.gz", lambda key: key in {"LICENSE", workflow}
                        or key.removeprefix("docs/").removesuffix(".md") in keys)
    pack.notices["LICENSE"] = sources.pop("LICENSE")
    deploy = sources.pop(workflow)
    for path, raw in sorted(sources.items()):
        key = path.removeprefix("docs/").removesuffix(".md")
        title, _ = markdown_article(raw, front_matter(raw, "title") or slug_title(Path(key)))
        body = prose_markdown(raw, pack.id).replace("<<< @/guide/static-deploy-github-pages.yaml", code_fence(deploy, "yaml"))
        body = transform_prose(body, vite_headings)
        slug = key.removesuffix("index") if key.endswith("/index") else key
        aliases = (key.rsplit("/", 1)[-1], title)
        if key.startswith("config/"):
            aliases += tuple(re.findall(r"^## ([A-Za-z][\w.]*) \{#", body, re.M))
        pack.add_markdown(key, title, body, base + slug, aliases)
    return pack


def vite_headings(prose):
    def heading(match):
        title = match[2]
        if re.search(r"\{#[\w.-]+\}\s*$", title):
            return match[0]
        anchor = re.sub(r"[^\w]+", "-", title.lower()).strip("-")
        return f"{match[1]} {title} {{#{anchor}}}" if anchor else match[0]

    return re.sub(r"^(#{1,6})[ \t]+([^\n]+)$", heading, prose, flags=re.M)


def download(cache, name):
    path = cache / name
    if path.is_file():
        return
    url = DOWNLOADS[name]
    request = Request(url, headers={"User-Agent": "Devline Docs Builder"})
    partial = path.with_suffix(path.suffix + ".part")
    with urlopen(request, timeout=120) as response, partial.open("wb") as output:
        shutil.copyfileobj(response, output)
    partial.replace(path)


def main():
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, required=True, help="directory containing the upstream archives and licenses")
    parser.add_argument("--reference", type=Path, required=True, help="prepared Offline Developer Kit/reference tree")
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parents[1] / "data")
    parser.add_argument("--fetch", action="store_true", help="download any missing upstream archives and licenses into --cache")
    parser.add_argument("--only", nargs="+", help="rebuild selected pack IDs")
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
    with tempfile.TemporaryDirectory(prefix="devline-docs-", dir=cache) as tmp:
        scratch = Path(tmp)
        def go_sources():
            goroot = options.go_root or Path(command("go", "env", "GOROOT").strip())
            chi = options.chi_root or Path(command("go", "env", "GOMODCACHE").strip()) / "github.com/go-chi/chi/v5@v5.5.0"
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
            "infrastructure": ((), lambda: infrastructure_pack(
                INFRASTRUCTURE_UPSTREAM, project / "data/infrastructure.json", scratch)),
            "node": (tuple(NODE_DOWNLOADS), lambda: node_pack(cache)),
            "typescript": (("typescript-docs-a3ed0be.tar.gz",), lambda: typescript_pack(cache)),
            "playwright": (("playwright-v1.65.0.tar.gz",), lambda: playwright_pack(cache)),
            "react": (("react-docs-2e6dca1.tar.gz",), lambda: react_pack(cache)),
            "vite": (("vite-v8.3.6.tar.gz",), lambda: vite_pack(cache)),
        }
        selected = set(options.only or builders)
        if not selected <= builders.keys():
            parser.error(f"unknown pack IDs: {sorted(selected - builders.keys())}")
        if options.fetch:
            names = set(name for key in selected for name in builders[key][0])
            with ThreadPoolExecutor(max_workers=4) as pool:
                list(pool.map(lambda name: download(cache, name), sorted(names)))
        catalog_path = data / "docs-catalog.json"
        entries = json.loads(read_text(catalog_path)) if catalog_path.is_file() else []
        done = set()
        for key, (_, build) in builders.items():
            if key not in selected or key in done:
                continue
            result = build()
            packs = result if isinstance(result, tuple) else (result,)
            for pack in packs:
                if pack.id in selected:
                    item = pack.emit(data)
                    entries = [entry for entry in entries
                               if (entry["id"], entry["version"]) != (pack.id, item["version"])]
                    first_release = next((index for index, entry in enumerate(entries)
                                          if entry["id"] == pack.id), len(entries))
                    entries.insert(first_release, item)
                    done.add(pack.id)
                    atomic_json(catalog_path, entries)


if __name__ == "__main__":
    main()
