#!/usr/bin/env python3
"""Audit shipped docset metadata, offline HTML, code examples, and local links."""

import argparse
import base64
from binascii import Error as Base64Error
from collections import Counter, defaultdict
from datetime import date
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
from urllib.parse import unquote, urljoin, urlsplit

from infrastructure_sources import REFERENCE as INFRASTRUCTURE_REFERENCE, UPSTREAM, example_text


FORBIDDEN_TAGS = {"base", "embed", "form", "iframe", "input", "link", "meta", "object", "script", "style"}
DIRECTIVE = re.compile(r"\[!INCLUDE(?:-|\[)|\[!code-[^\]]+\[|\{\*\s+[^\n]*docs_src/|^\s*:::(?:zone|moniker)\b", re.M | re.I)
IMAGE = re.compile(r"data:image/(?:gif|jpeg|png|svg\+xml|webp);base64,[A-Za-z0-9+/]+=*")
EXAMPLES = {
    ("python", "library/pathlib.html"): ("pathlib.Path.read_text", 2),
    ("fastapi", "tutorial/first-steps.md"): ("from fastapi import FastAPI", 1),
    ("go", "std/net/http/httputil"): ("ReverseProxy", 0),
    ("rust", "std/vec/struct.Vec.html"): ("method.push", 2),
    ("rails", "active_record_validations.md"): ("validates-associated", 2),
    ("node", "fs.html"): ("fspromisesreadfilepath-options", 2),
    ("typescript", "docs/handbook/2/narrowing.html"): ("typeof-type-guards", 2),
    ("playwright", "api/class-page.md"): ("page-goto", 2),
    ("react", "reference/react/useState"): ("updating-state-based-on-the-previous-state", 2),
    ("vite", "config/shared-options"): ("input", 1),
}
PYTHON_REFERENCE = "python@3.15.2+devline.3"
ASPNET_REFERENCE = "aspnet@2026.11.06+devline.5"
PILOT_LINKS = {
    (PYTHON_REFERENCE, "contents.html"): ("https://docs.python.org/3.15/library/index.html",),
    (PYTHON_REFERENCE, "library/audit_events.html"): (
        "https://docs.python.org/3.15/library/_thread.html#thread.start_new_thread",
    ),
    (ASPNET_REFERENCE, "web-api/index.md"): (
        "https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/parameter-binding#asparam7",
    ),
    (INFRASTRUCTURE_REFERENCE, "reference/postgres/queries-select-lists"): (
        "https://www.postgresql.org/docs/18/queries-table-expressions.html#QUERIES-TABLE-ALIASES",
    ),
    (INFRASTRUCTURE_REFERENCE, "reference/rabbitmq/tutorials/tutorial-one-dotnet"): (
        "https://www.rabbitmq.com/docs/4.3/networking#ports",
    ),
}
# Exact code from the pinned Python HTML, ASP.NET Markdown, and PostgreSQL HTML.
PILOT_CODE = {
    (PYTHON_REFERENCE, "library/pathlib.html"): ("python-repl", ">>> from pathlib import Path"),
    (ASPNET_REFERENCE, "web-api/index.md"): (
        "csharp", "if (!ModelState.IsValid)\n{\n    return BadRequest(ModelState);\n}",
    ),
    (INFRASTRUCTURE_REFERENCE, "reference/postgres/tutorial-select"): ("pgsql", "\nSELECT * FROM weather;"),
}
PILOT_PATHLIB = (PYTHON_REFERENCE, "library/pathlib.html")
# _images/pathlib-inheritance.png in the pinned Python 3.15.2 HTML archive.
PATHLIB_IMAGE_SHA256 = "4bd5db0b21f178fd8b16f7d999d0da20a00ca8d271cd556cfb1d26dea91aac88"


def source_key(url):
    parsed = urlsplit(url)
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        return None
    query = "?" + parsed.query if parsed.query else ""
    return parsed.hostname, unquote(parsed.path).rstrip("/") + query


class Article(HTMLParser):
    def __init__(self, capture=False):
        super().__init__(convert_charrefs=True)
        self.anchors = set()
        self.links = set()
        self.tags = Counter()
        self.errors = []
        self.pre_text = None
        self.pre_language = ""
        self.inline_code = 0
        self.examples = 0
        self.prose = []
        self.capture = capture
        self.code_blocks = []
        self.images = set()

    def handle_starttag(self, tag, attributes):
        self.tags[tag] += 1
        attrs = dict(attributes)
        if tag in FORBIDDEN_TAGS or any(key.lower().startswith("on") for key in attrs):
            self.errors.append(f"active markup <{tag}>")
        if tag == "img":
            image = attrs.get("src") or ""
            if not IMAGE.fullmatch(image):
                self.errors.append("non-offline image")
            elif self.capture and attrs.get("alt"):
                try:
                    content = base64.b64decode(image.partition(",")[2], validate=True)
                except Base64Error:
                    self.errors.append("invalid embedded image data")
                else:
                    self.images.add(hashlib.sha256(content).hexdigest())
        elif any(key in attrs for key in ("src", "srcset")):
            self.errors.append(f"resource attribute on <{tag}>")
        if "srcset" in attrs or "style" in attrs:
            self.errors.append(f"unexpected resource or style on <{tag}>")
        if anchor := attrs.get("id") or (attrs.get("name") if tag == "a" else None):
            self.anchors.add(anchor)
        if tag == "a" and (href := attrs.get("href")):
            scheme = urlsplit(href).scheme
            if scheme not in {"", "http", "https", "mailto"}:
                self.errors.append(f"unsupported link scheme {scheme}")
            else:
                self.links.add(href)
        if tag == "pre":
            if self.pre_text is not None:
                self.errors.append("nested example block")
            self.pre_text = []
            self.pre_language = ""
        if tag == "code":
            self.inline_code += 1
            if self.capture and self.pre_text is not None:
                self.pre_language = next((value.removeprefix("language-") for value in (attrs.get("class") or "").split()
                                          if value.startswith("language-")), "")

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag):
        if tag == "pre" and self.pre_text is not None:
            example = "".join(self.pre_text)
            if example.strip():
                self.examples += 1
                if self.capture:
                    self.code_blocks.append((self.pre_language, example.rstrip("\n")))
            else:
                self.errors.append("empty example block")
            self.pre_text = None
        if tag == "code":
            self.inline_code = max(0, self.inline_code - 1)

    def handle_data(self, text):
        if self.pre_text is not None:
            self.pre_text.append(text)
        elif not self.inline_code:
            self.prose.append(text)

    def close(self):
        super().close()
        if self.pre_text is not None:
            self.errors.append("unclosed example block")
        if DIRECTIVE.search("\n".join(self.prose)):
            self.errors.append("unresolved source directive")


def document_files(data, pack):
    directory = data / pack["path"]
    yield directory / "documents.json"
    for shard in range(2, pack.get("shards", 1) + 1):
        yield directory / f"documents-{shard}.json"


def rabbitmq_examples():
    root = UPSTREAM / "rabbitmq"
    manifest = json.loads((root / "manifest.json").read_text())
    return {
        (INFRASTRUCTURE_REFERENCE, "reference/rabbitmq/" + page["id"]):
        [("csharp", example_text(root, example).rstrip("\n")) for example in page["examples"]]
        for page in manifest["documents"] if page.get("examples")
    }


def check_pilot(articles, targets, rabbitmq):
    errors = []
    for (release, key), links in PILOT_LINKS.items():
        source, article = articles.get((release, key), ("", Article()))
        linked = {urljoin(source, href) for href in article.links}
        for link in links:
            fragment = unquote(urlsplit(link).fragment)
            destination = targets.get(source_key(link), {}).get(release)
            if link not in linked or destination is None or (fragment and fragment not in destination):
                errors.append(f"{release}:{key}: pilot link does not resolve in this release: {link}")
    for (release, key), example in PILOT_CODE.items():
        _, article = articles.get((release, key), ("", Article()))
        if example not in article.code_blocks:
            errors.append(f"{release}:{key}: pinned publisher code or its language hint changed")
    for (release, key), examples in rabbitmq.items():
        _, article = articles.get((release, key), ("", Article()))
        if Counter(examples) - Counter(article.code_blocks) or any(code == "loading..." for _, code in article.code_blocks):
            errors.append(f"{release}:{key}: missing or changed RabbitMQ publisher code")
    _, pathlib = articles.get(PILOT_PATHLIB, ("", Article()))
    if "pathlib.Path.read_text" not in pathlib.anchors or PATHLIB_IMAGE_SHA256 not in pathlib.images:
        errors.append(f"{PYTHON_REFERENCE}:library/pathlib.html: missing publisher API anchor or inheritance diagram")
    _, events = articles.get((PYTHON_REFERENCE, "library/audit_events.html"), ("", Article()))
    if not events.tags["table"] or not {"Audit event", "Arguments", "References"} <= set(events.prose):
        errors.append(f"{PYTHON_REFERENCE}:library/audit_events.html: missing publisher audit event table")
    _, query = articles.get((INFRASTRUCTURE_REFERENCE, "reference/postgres/tutorial-select"), ("", Article()))
    if not any(language == "plaintext" and "(1 row)" in code for language, code in query.code_blocks):
        errors.append(f"{INFRASTRUCTURE_REFERENCE}: PostgreSQL tutorial output lost its plain-text rendering")
    return errors


def audit(data, *, require_examples=True):
    catalog = json.loads((data / "docs-catalog.json").read_text())
    errors = []
    counts = Counter()
    packs = {}
    targets = defaultdict(dict)
    pending = set()
    articles = {}
    rabbitmq = rabbitmq_examples() if require_examples else {}
    pilot_keys = set(PILOT_LINKS) | set(PILOT_CODE) | set(rabbitmq) if require_examples else set()
    expected = set(EXAMPLES) if require_examples else set()
    for pack in catalog:
        pack_id, version = pack["id"], pack["version"]
        release = f"{pack_id}@{version}"
        totals = Counter()
        packs[release] = totals
        ids = set()
        if require_examples and not (data / pack["path"] / "SOURCES.md").is_file():
            errors.append(f"{release}: missing SOURCES.md")
        for field in ("upstream_version", "snapshot_date", "coverage", "omissions"):
            if not pack.get(field):
                errors.append(f"{release}: missing {field}")
        if pack.get("snapshot_date"):
            try:
                date.fromisoformat(pack["snapshot_date"])
            except ValueError:
                errors.append(f"{release}: invalid snapshot date")
        for shard in document_files(data, pack):
            for doc in json.loads(shard.read_text()):
                name = doc["id"]
                location = f"{release}:{name}"
                if name in ids:
                    errors.append(f"{location}: duplicate document ID")
                ids.add(name)
                totals["articles"] += 1
                key = source_key(doc["source_url"])
                if not key or not doc["body"].strip() or not doc["title"].strip():
                    errors.append(f"{location}: missing source, body, or title")
                if doc.get("category") not in {"guide", "api", "compatibility", "reference"}:
                    errors.append(f"{location}: invalid category")
                capture = (release, name) in pilot_keys
                article = Article(capture=capture)
                article.feed(doc.get("html") or "")
                article.close()
                if capture:
                    articles[(release, name)] = (doc["source_url"], article)
                errors.extend(f"{location}: {error}" for error in article.errors)
                if not article.tags:
                    errors.append(f"{location}: missing rendered HTML")
                aliases = set(doc.get("aliases", ()))
                for alias, anchor in doc.get("alias_anchors", {}).items():
                    if alias not in aliases or anchor not in article.anchors:
                        errors.append(f"{location}: invalid alias destination for {alias}")
                totals["examples"] += article.examples
                totals["tables"] += article.tags["table"]
                totals["embedded_images"] += article.tags["img"]
                totals["anchors"] += len(article.anchors)
                targets[key].setdefault(release, set()).update(article.anchors)
                for href in article.links:
                    parsed = urlsplit(urljoin(doc["source_url"], href))
                    target = source_key(parsed.geturl())
                    if target:
                        pending.add((release, name, key, target, unquote(parsed.fragment)))
                probe = (pack_id, name)
                if probe in EXAMPLES:
                    needle, examples = EXAMPLES[probe]
                    if ((needle in doc["body"] or needle in article.anchors)
                            and article.examples >= examples):
                        expected.discard(probe)
        if pack.get("entry_id") and pack["entry_id"] not in ids:
            errors.append(f"{release}: missing publisher contents article {pack['entry_id']}")
        counts.update(totals)
    missing = []
    for release, name, source, target, fragment in sorted(pending):
        local = targets.get(target)
        if not local:
            counts["links_outside_snapshot"] += 1
            continue
        counts["local_links"] += 1
        # A document from this exact release always wins over shared URLs
        # in other releases. Cross-pack links can match another installed pack.
        found = local.get(release)
        anchor_exists = fragment in found if found is not None else any(fragment in ids for ids in local.values())
        if fragment and not anchor_exists:
            missing.append({"release": release, "document": name, "target": "".join(target),
                            "fragment": fragment, "same_page": source == target})
    errors.extend(f"Missing source example probe {pack}:{doc}" for pack, doc in sorted(expected))
    if require_examples:
        errors.extend(check_pilot(articles, targets, rabbitmq))
    counts["missing_local_anchors"] = len(missing)
    counts["missing_same_page_anchors"] = sum(link["same_page"] for link in missing)
    return {"counts": dict(counts), "releases": packs, "errors": errors, "missing_anchors": missing}


def anchor_key(link):
    return link["release"], link["document"], link["target"], link["fragment"]


def regressions(report, baseline):
    known_errors = set(baseline.get("render_errors", []))
    known_anchors = {tuple(item) for item in baseline.get("missing_anchors", [])}
    known_omissions = {tuple(item) for item in baseline.get("known_source_omissions", [])}
    observed = {anchor_key(link) for link in report["missing_anchors"]}
    stale_omissions = sorted(item for item in known_omissions - observed if item[0] in report["releases"])
    return (
        [error for error in report["errors"] if error not in known_errors]
        + [f"{release}:{document}: recheck known omission {target}#{fragment}"
           for release, document, target, fragment in stale_omissions],
        [link for link in report["missing_anchors"] if anchor_key(link) not in known_anchors | known_omissions],
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, default=Path(__file__).resolve().parents[1] / "data")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--baseline", type=Path, default=Path(__file__).with_name("doc-audit-baseline.json"))
    options = parser.parse_args()
    report = audit(options.data)
    baseline = json.loads(options.baseline.read_text())
    errors, anchors = regressions(report, baseline)
    omissions = {tuple(item) for item in baseline.get("known_source_omissions", [])}
    report["known_source_omissions"] = [link for link in report["missing_anchors"] if anchor_key(link) in omissions]
    report["regressions"] = {"errors": errors, "missing_anchors": anchors}
    if options.output:
        options.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"counts": report["counts"], "known_source_omissions": len(report["known_source_omissions"]),
                      "known_render_errors": len(set(report["errors"]) & set(baseline.get("render_errors", []))),
                      "new_render_errors": len(errors), "new_missing_anchors": len(anchors)}, indent=2))
    for error in errors[:20]:
        print(error)
    for link in anchors[:20]:
        print(f"{link['release']}:{link['document']}: missing {link['target']}#{link['fragment']}")
    if errors or anchors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
