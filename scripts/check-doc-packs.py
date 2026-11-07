#!/usr/bin/env python3
"""Audit shipped docset metadata, offline HTML, code examples, and local links."""

import argparse
from collections import Counter, defaultdict
from datetime import date
from html.parser import HTMLParser
import json
from pathlib import Path
import re
from urllib.parse import unquote, urljoin, urlsplit


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


def source_key(url):
    parsed = urlsplit(url)
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        return None
    return parsed.hostname, unquote(parsed.path).rstrip("/")


class Article(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.anchors = set()
        self.links = set()
        self.tags = Counter()
        self.errors = []
        self.pre_text = None
        self.inline_code = 0
        self.examples = 0
        self.prose = []

    def handle_starttag(self, tag, attributes):
        self.tags[tag] += 1
        attrs = dict(attributes)
        if tag in FORBIDDEN_TAGS or any(key.lower().startswith("on") for key in attrs):
            self.errors.append(f"active markup <{tag}>")
        if tag == "img":
            if not IMAGE.fullmatch(attrs.get("src") or ""):
                self.errors.append("non-offline image")
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
        if tag == "code":
            self.inline_code += 1

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag):
        if tag == "pre" and self.pre_text is not None:
            if "".join(self.pre_text).strip():
                self.examples += 1
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


def audit(data, *, require_examples=True):
    catalog = json.loads((data / "docs-catalog.json").read_text())
    errors = []
    counts = Counter()
    packs = {}
    targets = defaultdict(dict)
    pending = set()
    expected = set(EXAMPLES) if require_examples else set()
    for pack in catalog:
        pack_id, version = pack["id"], pack["version"]
        release = f"{pack_id}@{version}"
        totals = Counter()
        packs[release] = totals
        ids = set()
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
                article = Article()
                article.feed(doc.get("html") or "")
                article.close()
                errors.extend(f"{location}: {error}" for error in article.errors)
                if not article.tags:
                    errors.append(f"{location}: missing rendered HTML")
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
    counts["missing_local_anchors"] = len(missing)
    counts["missing_same_page_anchors"] = sum(link["same_page"] for link in missing)
    return {"counts": dict(counts), "releases": packs, "errors": errors, "missing_anchors": missing}


def regressions(report, baseline):
    known_errors = set(baseline.get("render_errors", []))
    known_anchors = {tuple(item) for item in baseline.get("missing_anchors", [])}
    return (
        [error for error in report["errors"] if error not in known_errors],
        [link for link in report["missing_anchors"]
         if (link["release"], link["document"], link["target"], link["fragment"]) not in known_anchors],
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
    report["regressions"] = {"errors": errors, "missing_anchors": anchors}
    if options.output:
        options.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"counts": report["counts"], "known_render_errors": len(report["errors"]) - len(errors),
                      "new_render_errors": len(errors), "new_missing_anchors": len(anchors)}, indent=2))
    for error in errors[:20]:
        print(error)
    for link in anchors[:20]:
        print(f"{link['release']}:{link['document']}: missing {link['target']}#{link['fragment']}")
    if errors or anchors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
