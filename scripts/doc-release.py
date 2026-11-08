#!/usr/bin/env python3
"""Build and verify versioned Devline docset release assets and their channel manifest."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import tarfile
import tempfile
from urllib.parse import urlsplit

from reference_io import atomic_json


MAX_FILE = 128 * 1024 * 1024
MAX_TOTAL = 1024 * 1024 * 1024
MAX_ARCHIVE = 512 * 1024 * 1024
MAX_FILES = 10_000
MANIFEST_NAME = "docs-manifest.json"
RETIRED_FROM_CHANNEL = {("infrastructure", "2026.11.07+devline.1")}


def digest(file):
    with file.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def identity(pack):
    pack_id, version = pack["id"], pack["version"]
    if pack_id == "swiftui" or pack.get("local_only", False) is not False:
        raise ValueError("Local SDK documentation must be imported on the destination Mac, not published as a docset archive")
    if not re.fullmatch(r"[a-z][a-z0-9_-]{0,63}", pack_id) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.+_-]{0,95}", version):
        raise ValueError(f"Invalid docset identity {pack_id}@{version}")
    if pack["path"] != f"docs/{pack_id}/{version}":
        raise ValueError(f"Docset path must match {pack_id}@{version}")
    return pack_id, version


def archive_name(pack_id, version):
    # Use GitHub's stable asset alphabet and keep '+' distinct from '_'.
    return f"{pack_id}-{version.replace('_', '_u').replace('+', '_p')}.tar.gz"


def require_https(url):
    parsed = urlsplit(url)
    if (len(url) > 2048 or parsed.scheme != "https" or not parsed.hostname
            or parsed.username is not None or parsed.password is not None or "?" in url or "#" in url):
        raise ValueError("Use an HTTPS release URL without credentials, query, or fragment")
    if "/releases/latest/" in url:
        raise ValueError("Use a versioned release asset URL")
    return parsed


def safe_path(name):
    path = PurePosixPath(name)
    if (not name or len(name.encode()) > 1024 or path.is_absolute() or "\\" in name or ":" in name
            or any(part in {"", ".", "..", ".devline-pack.json"} for part in name.split("/"))
            or any(ord(char) < 32 or ord(char) == 127 for char in name)):
        raise ValueError(f"Unsafe docset file path {name!r}")
    return name


def required_shards(pack):
    count = pack.get("shards", 1)
    if type(count) is not int or not 1 <= count <= 1024:
        raise ValueError("Invalid docset shard count")
    return {"documents.json", *(f"documents-{index}.json" for index in range(2, count + 1))}


def create_archive(source, output, pack):
    if not source.is_dir() or source.is_symlink():
        raise ValueError(f"Missing docset directory {source}")
    total = 0
    seen = set()
    with tempfile.NamedTemporaryFile(dir=output.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            with gzip.GzipFile(filename="", fileobj=stream, mode="wb", mtime=0, compresslevel=9) as gzip_stream:
                with tarfile.open(fileobj=gzip_stream, mode="w|", format=tarfile.USTAR_FORMAT) as archive:
                    for file in sorted(source.rglob("*")):
                        if file.is_symlink():
                            raise ValueError(f"Links are unsupported in docsets: {file}")
                        if file.is_dir():
                            continue
                        name = safe_path(file.relative_to(source).as_posix())
                        info = tarfile.TarInfo(name)
                        info.size = file.stat().st_size
                        info.mode = 0o644
                        total += info.size
                        seen.add(name)
                        if not file.is_file() or len(seen) > MAX_FILES or info.size > MAX_FILE or total > MAX_TOTAL:
                            raise ValueError(f"Docset exceeds archive limits: {file}")
                        with file.open("rb") as data:
                            archive.addfile(info, data)
            stream.flush()
            if not required_shards(pack) | {"SOURCES.md"} <= seen:
                raise ValueError(f"Docset archive has missing shards or source attribution: {source}")
            if temporary.stat().st_size > MAX_ARCHIVE:
                raise ValueError("Compressed docset exceeds release limits")
            temporary.replace(output)
        finally:
            temporary.unlink(missing_ok=True)


def default_notes(pack):
    coverage = ". ".join(item.rstrip(".") for item in pack.get("coverage", [])) or pack["description"].rstrip(".")
    return f"{pack['name']} offline snapshot. {coverage}."


def build(data, output, base, selected=None, previous=None, notes=None):
    require_https(base)
    catalog = json.loads((data / "docs-catalog.json").read_text())
    catalog = [pack for pack in catalog if (pack["id"], pack["version"]) not in RETIRED_FROM_CHANNEL]
    known_ids = {pack["id"] for pack in catalog}
    exact = {f"{pack['id']}@{pack['version']}" for pack in catalog}
    references = set(selected or known_ids)
    unknown = references - known_ids - exact
    if unknown:
        raise ValueError(f"Unknown documentation pack references: {sorted(unknown)}")
    selected_packs = [pack for pack in catalog if pack["id"] in references
                      or f"{pack['id']}@{pack['version']}" in references]
    selected_releases = {identity(pack): pack for pack in selected_packs}
    if len(selected_releases) != len(selected_packs):
        raise ValueError("Multiple catalog entries select the same docset release")
    releases = {}
    if previous:
        prior = json.loads(previous.read_text())
        if prior["schema_version"] != 1:
            raise ValueError("Unsupported prior documentation manifest")
        for release in prior["releases"]:
            key = identity(release["pack"])
            if key in RETIRED_FROM_CHANNEL:
                continue
            if key in releases:
                raise ValueError(f"Duplicate prior release {key[0]}@{key[1]}")
            release_asset(release, *key)
            releases[key] = release
    identities = releases.keys() | selected_releases.keys()
    if len({archive_name(*key) for key in identities}) != len(identities):
        raise ValueError("Multiple docset releases have the same asset name")
    output.mkdir(parents=True, exist_ok=True)
    for (pack_id, version), pack in selected_releases.items():
        filename = archive_name(pack_id, version)
        asset = output / filename
        source = data
        for part in PurePosixPath(pack["path"]).parts:
            source = source / part
            if source.is_symlink():
                raise ValueError(f"Links are unsupported in docset paths: {source}")
        if not source.resolve().is_relative_to(data.resolve()):
            raise ValueError(f"Docset path escapes the catalog: {source}")
        with tempfile.TemporaryDirectory(dir=output) as staging:
            candidate = Path(staging) / filename
            create_archive(source, candidate, pack)
            checksum = digest(candidate)
            old = releases.get((pack_id, version))
            if old and old["sha256"] != checksum:
                raise ValueError(f"Immutable released content changed for {pack_id}@{version}")
            candidate.replace(asset)
        if not old:
            releases[pack_id, version] = {
                "pack": pack,
                "release_notes": (notes or {}).get(f"{pack_id}@{version}") or (notes or {}).get(pack_id) or default_notes(pack),
                "url": base.rstrip("/") + "/" + filename,
                "size": asset.stat().st_size,
                "sha256": checksum,
            }
        print(f"{filename:<58} {asset.stat().st_size:>10} bytes {checksum}", flush=True)
    manifest = {"schema_version": 1, "releases": [release for _, release in sorted(releases.items())]}
    manifest_path = output / MANIFEST_NAME
    atomic_json(manifest_path, manifest)
    return manifest_path


def verify(directory, *, allow_missing=False):
    manifest = json.loads((directory / MANIFEST_NAME).read_text())
    if manifest["schema_version"] != 1:
        raise ValueError("Unsupported documentation release manifest")
    count = 0
    seen_releases = set()
    seen_assets = set()
    for release in manifest["releases"]:
        pack = release["pack"]
        pack_id, version = identity(pack)
        if (pack_id, version) in seen_releases:
            raise ValueError(f"Duplicate release {pack_id}@{version}")
        seen_releases.add((pack_id, version))
        filename = release_asset(release, pack_id, version)
        if filename in seen_assets:
            raise ValueError(f"Duplicate release asset {filename}")
        seen_assets.add(filename)
        path = directory / filename
        if allow_missing and not path.exists():
            continue
        if path.stat().st_size != release["size"] or digest(path) != release["sha256"]:
            raise ValueError(f"Archive checksum or size mismatch: {filename}")
        seen = set()
        total = 0
        with tarfile.open(path, "r|gz") as archive:
            for entry in archive:
                name = safe_path(entry.name)
                total += entry.size
                if (not entry.isfile() or name in seen or len(seen) >= MAX_FILES
                        or entry.size > MAX_FILE or total > MAX_TOTAL):
                    raise ValueError(f"Unsafe or oversized archive entry: {name}")
                seen.add(name)
        if not required_shards(pack) | {"SOURCES.md"} <= seen:
            raise ValueError(f"Archive is incomplete: {filename}")
        count += 1
    print(f"Verified {count} docset release archives")


def release_asset(release, pack_id, version):
    url = require_https(release["url"])
    filename = archive_name(pack_id, version)
    size = release["size"]
    if (not url.path.endswith("/" + filename) or type(size) is not int or not 1 <= size <= MAX_ARCHIVE
            or not re.fullmatch(r"[0-9a-f]{64}", release["sha256"])
            or not release["release_notes"].strip()):
        raise ValueError(f"Invalid release metadata for {pack_id}@{version}")
    return filename


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)
    builder = subcommands.add_parser("build")
    builder.add_argument("--data", type=Path, default=Path(__file__).resolve().parents[1] / "data")
    builder.add_argument("--output", required=True, type=Path)
    builder.add_argument("--asset-base", required=True)
    builder.add_argument("--only", nargs="+", metavar="PACK[@VERSION]", help="build only these releases")
    builder.add_argument("--previous-manifest", type=Path, help="keep previously released assets in the channel")
    builder.add_argument("--notes", type=Path, help="JSON notes keyed by pack ID or pack@version")
    checker = subcommands.add_parser("verify")
    checker.add_argument("--output", required=True, type=Path)
    checker.add_argument("--allow-missing-prior-assets", action="store_true")
    options = parser.parse_args()
    if options.command == "build":
        notes = json.loads(options.notes.read_text()) if options.notes else None
        print(build(options.data, options.output, options.asset_base, options.only, options.previous_manifest, notes))
    else:
        verify(options.output, allow_missing=options.allow_missing_prior_assets)


if __name__ == "__main__":
    main()
