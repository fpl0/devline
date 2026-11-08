#!/usr/bin/env python3
"""Import personal DevDocs snapshots using Devline's local pack installer."""

import argparse
from collections import defaultdict
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path, PurePosixPath
import posixpath
import re
import shutil
import subprocess
import sys
import tarfile
from tempfile import TemporaryDirectory
from urllib.parse import quote, unquote, urlsplit, urlunsplit
from urllib.request import Request, urlopen

from reference_io import atomic_json, write_shards
from reference_rendering import (
    MAX_IMAGE_BYTES, DocumentParser, as_text, asset_path, clean_text, file_assets, prepared_html,
)


MANIFEST_URL = "https://devdocs.io/docs.json"
ARCHIVE_URL = "https://downloads.devdocs.io/"
SLUG = re.compile(r"[a-z0-9][a-z0-9_.-]*(?:~[a-z0-9][a-z0-9_.-]*)?")
VERSION_PREFIX = re.compile(r"v?([0-9]+(?:\.[0-9]+)*)", re.IGNORECASE)
INPUT_FILES = {"meta.json", "index.json", "db.json"}
MAX_DB_BYTES = 512 * 1024 * 1024
MAX_ARCHIVE_BYTES = 1024 * 1024 * 1024
MAX_IMAGES_BYTES = 32 * 1024 * 1024


def get(url):
    # The CDN serves its public archives to browser clients.
    return urlopen(Request(url, headers={"User-Agent": "Mozilla/5.0"}), timeout=45)


def json_input(stream, name, limit):
    data = stream.read(limit + 1)
    if len(data) > limit:
        raise ValueError(f"{name} exceeds its input size limit")
    return json.loads(data)


def read_json(path, limit=MAX_DB_BYTES):
    with path.open("rb") as stream:
        return json_input(stream, path.name, limit)


def archive_input(path):
    contents, images = {}, {}
    image_bytes = 0
    unpacked = 0
    with tarfile.open(path, "r|gz") as archive:
        for member in archive:
            unpacked += member.size + 512
            if unpacked > 2 * MAX_ARCHIVE_BYTES:
                raise ValueError("Archive exceeds its unpacked size limit")
            name = PurePosixPath(member.name)
            if not member.isfile() or name.is_absolute() or ".." in name.parts:
                continue
            if len(name.parts) == 1 and name.name in INPUT_FILES:
                if name.name in contents:
                    raise ValueError(f"Duplicate {name} in {path.name}")
                limit = MAX_DB_BYTES if name.name == "db.json" else 64 * 1024 * 1024
                if member.size > limit:
                    raise ValueError(f"{name} exceeds its input size limit")
                with archive.extractfile(member) as stream:
                    contents[name.name] = json_input(stream, name.name, limit)
            elif (member.size <= MAX_IMAGE_BYTES and image_bytes + member.size <= MAX_IMAGES_BYTES
                  and asset_path(str(name)) and str(name) not in images):
                with archive.extractfile(member) as stream:
                    images[str(name)] = stream.read()
                image_bytes += member.size
    if not {"db.json", "index.json"} <= contents.keys():
        raise ValueError("Archive must contain db.json and index.json at its root")

    def assets(key):
        def load(raw):
            path = asset_path(raw)
            if path:
                relative = path.lstrip("/") if path.startswith("/") else posixpath.join(posixpath.dirname(key), path)
                return images.get(posixpath.normpath(relative))
            return None
        return load

    return contents.get("meta.json", {}), contents["index.json"], contents["db.json"], assets


def extracted_input(path):
    meta = path / "meta.json"
    return (read_json(meta) if meta.is_file() else {}, read_json(path / "index.json"),
            read_json(path / "db.json"), lambda key: file_assets(path / key, path))


def manifest_rows(value):
    if not isinstance(value, list):
        raise ValueError("DevDocs manifest must contain a list of docsets")
    rows = {}
    for entry in value:
        slug = entry.get("slug") if isinstance(entry, dict) else None
        if not isinstance(slug, str) or not SLUG.fullmatch(slug):
            raise ValueError(f"Invalid docset slug: {slug!r}")
        # DevDocs sometimes lists two releases under one unversioned archive
        # URL. The archive's own meta.json identifies the downloadable release.
        previous = rows.get(slug)
        if previous is None or (entry.get("mtime") or 0) > (previous.get("mtime") or 0):
            rows[slug] = entry
    return rows


def local_inputs(source):
    if source.is_file() and source.name.endswith(".tar.gz"):
        return {}, {source.name.removesuffix(".tar.gz"): source}
    if not source.is_dir():
        raise ValueError(f"Expected a DevDocs checkout, docs directory, or .tar.gz archive: {source}")
    if (source / "public/docs").is_dir():
        source = source / "public/docs"
    if (source / "db.json").is_file():
        return {}, {source.name: source}
    manifest = source / "docs.json"
    rows = manifest_rows(read_json(manifest)) if manifest.is_file() else {}
    paths = {}
    for path in sorted(source.iterdir()):
        slug = path.name.removesuffix(".tar.gz")
        if not SLUG.fullmatch(slug):
            continue
        if path.is_dir() and (path / "db.json").is_file():
            paths[slug] = path
        elif path.is_file() and path.name.endswith(".tar.gz"):
            paths.setdefault(slug, path)
    return rows, paths


def local_selection_metadata(manifest, paths):
    metadata = dict(manifest)
    for slug, path in paths.items():
        if not path.is_dir():
            continue
        try:
            row = read_json(path / "meta.json", 1024 * 1024)
        except (OSError, ValueError):
            continue
        if isinstance(row, dict) and row.get("slug", slug) == slug:
            metadata[slug] = {**metadata.get(slug, {}), **row}
    return metadata


def snapshot_order(slug, row):
    _, _, edition = slug.partition("~")
    version = ()
    for value in (row.get("release"), row.get("version"), edition):
        match = VERSION_PREFIX.match(str(value or "").strip())
        if match:
            parts = [int(part) for part in match[1].split(".")]
            while len(parts) > 1 and parts[-1] == 0:
                parts.pop()
            version = tuple(parts)
            break
    # Some families publish their current docs at an unversioned slug without
    # declaring a release, while retaining older archives with numbered slugs.
    canonical = not edition
    mtime = row.get("mtime")
    return canonical and not version, version, canonical, mtime if isinstance(mtime, int) else 0, slug


def selected_slugs(available, requested, latest=False, metadata=None):
    if not requested and not latest:
        return list(available)
    families = defaultdict(list)
    for slug in available:
        families[slug.partition("~")[0]].append(slug)
    metadata = metadata or {}
    selected = {}
    for slug in dict.fromkeys(requested or families):
        if slug in available and (not latest or "~" in slug):
            selected[slug] = None
            continue
        versions = families.get(slug)
        if not versions:
            raise ValueError(f"DevDocs docset {slug!r} was not found")
        if latest:
            newest = max(versions, key=lambda item: snapshot_order(item, metadata.get(item, {})))
            selected[newest] = None
        else:
            selected.update(dict.fromkeys(versions))
    return list(selected)


def download_archive(slug, destination):
    url = ARCHIVE_URL + quote(slug, safe="~") + ".tar.gz"
    with get(url) as response, destination.open("wb") as output:
        remaining = MAX_ARCHIVE_BYTES
        while chunk := response.read(1024 * 1024):
            remaining -= len(chunk)
            if remaining < 0:
                raise ValueError("Downloaded archive exceeds its size limit")
            output.write(chunk)


def web_source(value):
    if not value or re.search(r"[\x00-\x20\\]", value):
        return None
    try:
        url = urlsplit(value)
        if url.scheme in {"https", "http"} and url.hostname and url.username is None:
            return url._replace(fragment="").geturl()
    except ValueError:
        pass
    return None


def path_keys(path):
    path = path.lstrip("/")
    if path in {"", ".", "./"}:
        path = "index"
    elif path.endswith("/"):
        path += "index"
    path = path.removesuffix(".html").removesuffix(".md")
    yield path
    if path != unquote(path):
        yield unquote(path)
    if not path.endswith("/index"):
        yield path + "/index"


def page_key(path, pages):
    return next((key for key in path_keys(path) if key in pages), None)


def offline_href(raw, key, sources):
    if not raw or raw.startswith(("#", "//")):
        return None
    try:
        link = urlsplit(raw)
    except ValueError:
        return None
    if link.scheme or link.netloc:
        return None
    path = link.path
    if path and not path.startswith("/"):
        relative = posixpath.normpath(posixpath.join(posixpath.dirname(key), path))
        path = relative + "/" if path.endswith("/") else relative
    target = page_key(path, sources) if path else key
    if target is None:
        return None
    return urlunsplit(urlsplit(sources[target])._replace(fragment=link.fragment))


def page_tree(raw):
    parser = DocumentParser()
    parser.feed(raw)
    parser.close()
    return parser.root


def source_of(raw):
    # DevDocs appends this attribution at the end of each page. Parse only
    # that tail here, then parse the whole article once when rendering.
    start = raw.rfind('class="_attribution"')
    if start == -1:
        start = raw.rfind("_attribution-link")
        start = raw.rfind("<a", 0, start) if start != -1 else -1
    if start != -1:
        for node in page_tree(raw[start:]).walk():
            if node.tag == "a" and "_attribution-link" in (node.attrs.get("class") or "").split():
                return web_source(node.attrs.get("href"))
    return None


def articles(pages, index, base, pack_id, assets):
    if not isinstance(pages, dict) or not pages or not isinstance(index, dict):
        raise ValueError("Invalid or empty DevDocs page database or index")
    sources = {}
    for key, raw in pages.items():
        if (not key or len(key) > 4096 or key.startswith("/") or ".." in PurePosixPath(key).parts
                or re.search(r"[\x00-\x1f\\]", key) or not isinstance(raw, str)):
            raise ValueError(f"Invalid DevDocs page key or content: {key!r}")
        sources[key] = source_of(raw) or base + quote(key, safe="/")
    aliases = defaultdict(dict)
    for entry in index.get("entries", []):
        name, path = entry.get("name"), entry.get("path")
        if isinstance(name, str) and name.strip() and isinstance(path, str):
            # DevDocs paths may contain literal '?' in symbol names; the page
            # database key, if present, is authoritative.
            plain, _, anchor = path.partition("#")
            key = page_key(plain, sources) or page_key(plain.partition("?")[0], sources)
            if key is not None:
                aliases[key][name.strip()] = bool(anchor)
    for key in sorted(pages):
        root = page_tree(pages[key])
        heading = next((node for node in root.walk() if node.tag in {"h1", "title"}), None)
        title = clean_text(as_text(heading)) if heading is not None else ""
        names = aliases[key]
        if not title:
            title = next((name for name, anchored in names.items() if not anchored), "")
        for node in root.walk():
            if node.tag == "a":
                destination = offline_href(node.attrs.get("href"), key, sources)
                if destination:
                    node.attrs["href"] = destination
        body, rendered = prepared_html(root, sources[key], pack_id, assets=assets(key))
        yield {"id": key, "title": title or key, "body": body, "html": rendered,
               "source_url": sources[key], "aliases": sorted(names)}


def install_snapshot(path, slug, manifest, workspace, args):
    meta, index, pages, assets = extracted_input(path) if path.is_dir() else archive_input(path)
    meta = {**manifest, **meta}
    slug = meta.get("slug", slug)
    if not isinstance(slug, str) or not SLUG.fullmatch(slug):
        raise ValueError(f"Invalid DevDocs docset slug: {slug!r}")
    if manifest and slug != manifest["slug"]:
        raise ValueError("Docset metadata does not match its catalog slug")
    family, _, edition = slug.partition("~")
    pack_id = "devdocs-" + family.replace(".", "_")
    if len(pack_id) > 64:
        raise ValueError("DevDocs docset slug is too long")
    base = f"https://devdocs.io/{quote(slug, safe='~')}/"
    output = workspace / "pack"
    output.mkdir()
    shards, _ = write_shards(output, articles(pages, index, base, pack_id, assets))
    version_hint = meta.get("release") or meta.get("version")
    if not version_hint and VERSION_PREFIX.match(edition):
        version_hint = edition
    source_release = str(version_hint or "snapshot")
    release = re.sub(r"[^a-zA-Z0-9._-]", "-", source_release).strip("._-")[:40] or "snapshot"
    title = str(meta.get("name") or family)
    notice = meta.get("attribution") or "See the attribution supplied with each article where available."
    notice = clean_text(as_text(page_tree(notice))) if isinstance(notice, str) else ""
    sources = (
        f"# {title}\n\nPersonal offline snapshot generated from [DevDocs]({base}).\n\n"
        f"Catalog slug: `{slug}`\n\nSource release: `{source_release}`\n\n"
        "Each article retains publisher attribution where supplied by the snapshot. "
        "Devline sanitizes markup and rewrites links between included articles.\n\n"
        f"{notice}\n"
    )
    (output / "SOURCES.md").write_text(sources, encoding="utf-8")
    digest = hashlib.sha256()
    for file in sorted(output.iterdir()):
        digest.update(file.name.encode())
        with file.open("rb") as stream:
            digest.update(file.stat().st_size.to_bytes(8, "big"))
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
    version = f"{release}+devdocs.{digest.hexdigest()}"
    relative = f"docs/{pack_id}/{version}"
    target = workspace / relative
    target.parent.mkdir(parents=True)
    output.rename(target)
    pack = {"id": pack_id, "name": f"DevDocs · {title}", "stack": pack_id,
            "version": version, "description": f"Personal DevDocs snapshot of {title}",
            "source_url": base, "license": "See the original publisher notices in this snapshot",
            "path": relative, "local_only": True, "shards": shards,
            "coverage": ["Articles and symbol index provided by this DevDocs snapshot"],
            "omissions": ["Remote assets, interactive content and pages absent from DevDocs"]}
    if version_hint:
        pack["upstream_version"] = source_release
    if isinstance(meta.get("mtime"), int) and 0 <= meta["mtime"] <= 253402300799:
        pack["snapshot_date"] = datetime.fromtimestamp(meta["mtime"], timezone.utc).date().isoformat()
    atomic_json(workspace / "catalog.json", {"schema_version": 1, "docs": [pack], "templates": []})
    command = [args.devline, "--offline", "--json", "--catalog", str(workspace)]
    if args.home:
        command.extend(["--home", str(args.home)])
    command.extend(["docs", "import-local", pack_id, "--version", version])
    if args.latest:
        command.append("--keep-latest")
    result = subprocess.run(command, text=True, capture_output=True, check=False)
    if result.returncode:
        raise RuntimeError(result.stderr.strip() or result.stdout.strip() or "Devline could not install this snapshot")
    return {"slug": slug, **json.loads(result.stdout)["data"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", type=Path, help="DevDocs checkout, docs directory, or .tar.gz archive")
    parser.add_argument("--online", action="store_true", help="Download from the DevDocs catalog")
    parser.add_argument("--doc", action="append", default=[], help="Import a docset slug or family (repeatable)")
    parser.add_argument("--latest", action="store_true",
                        help="Import one snapshot per family, while honoring explicitly versioned slugs")
    parser.add_argument("--devline", default="devline", help="Path to the Devline CLI")
    parser.add_argument("--home", type=Path, help="Use a separate Devline library")
    parser.add_argument("--json", action="store_true", help="Emit the import results as JSON")
    args = parser.parse_args()
    if args.online == (args.source is not None):
        parser.error("Choose either SOURCE or --online")
    if not shutil.which(args.devline):
        parser.error("Devline CLI not found. Install it or pass --devline PATH.")
    try:
        if args.online:
            with get(MANIFEST_URL) as response:
                manifest = manifest_rows(json_input(response, "DevDocs manifest", 16 * 1024 * 1024))
            paths = {}
        else:
            manifest, paths = local_inputs(args.source.expanduser())
        metadata = local_selection_metadata(manifest, paths) if args.latest and not args.online else manifest
        slugs = selected_slugs(manifest if args.online else paths, args.doc, args.latest, metadata)
        if not slugs:
            raise ValueError("No DevDocs snapshots found")
    except (OSError, ValueError, TypeError) as error:
        parser.exit(1, f"DevDocs import: {error}\n")

    installed, skipped, failed = [], [], []
    for number, slug in enumerate(slugs, 1):
        try:
            with TemporaryDirectory(prefix="devline-devdocs-") as scratch:
                workspace = Path(scratch)
                if args.online:
                    path = workspace / "download.tar.gz"
                    download_archive(slug, path)
                else:
                    path = paths[slug]
                item = install_snapshot(path, slug, manifest.get(slug, {}), workspace, args)
                if item.get("skipped"):
                    skipped.append(item)
                    if not args.json:
                        print(f"[{number}/{len(slugs)}] {item['slug']}: kept {item['version']}", flush=True)
                else:
                    installed.append(item)
                    if not args.json:
                        print(f"[{number}/{len(slugs)}] {item['slug']}: {item['count']} articles", flush=True)
        except (OSError, ValueError, TypeError, AttributeError, KeyError, RuntimeError, tarfile.TarError) as error:
            failed.append({"slug": slug, "error": str(error)})
            print(f"[{number}/{len(slugs)}] {slug}: {error}", file=sys.stderr, flush=True)
    if args.json:
        print(json.dumps({"installed": installed, "skipped": skipped, "failed": failed}, ensure_ascii=False))
    else:
        summary = f"Imported {len(installed)} docsets. {len(failed)} failed."
        if skipped:
            summary += f" Kept {len(skipped)} newer installed docsets."
        print(summary)
    return int(bool(failed))


if __name__ == "__main__":
    sys.exit(main())
