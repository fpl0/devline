"""Read pinned publisher documentation inputs for the infrastructure docset."""

import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import tarfile
from urllib.parse import quote, urlsplit


UPSTREAM = Path(__file__).resolve().parent / "infrastructure-docs" / "upstream"
VERSION = "2026.11.07"
REVISION = "devline.3"
REFERENCE = f"infrastructure@{VERSION}+{REVISION}"
DOCUMENT_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]*(?:/[A-Za-z0-9][A-Za-z0-9_.-]*)*")


def document_id(service_id, article_id):
    return f"reference/{service_id}/{article_id}"


def source_path(root, path):
    candidate = (root / path).resolve()
    if not candidate.is_relative_to(root.resolve()) or not candidate.is_file():
        raise ValueError(f"Missing or unsafe documentation source: {root / path}")
    return candidate


def archive_documents(root, source):
    path = source_path(root, source["path"])
    with path.open("rb") as stream:
        if hashlib.file_digest(stream, "sha256").hexdigest() != source["sha256"]:
            raise ValueError(f"Publisher archive checksum changed: {path}")
    prefix = PurePosixPath(source["root"])
    with tarfile.open(path) as archive:
        documents = []
        for member in sorted(archive, key=lambda item: item.name):
            name = PurePosixPath(member.name)
            if (member.isfile() and name.is_relative_to(prefix) and not name.is_absolute()
                    and ".." not in name.parts and name.suffix == ".html"):
                relative = name.relative_to(prefix)
                documents.append({
                    "id": relative.with_suffix("").as_posix(),
                    "path": name.as_posix(),
                    "source_url": source["url"] + quote(relative.as_posix()),
                    "selector": source.get("selector"),
                })
    if len(documents) != source["count"]:
        raise ValueError(f"Unexpected publisher archive page count: {path}")
    return documents


def read_sources(services, upstream=UPSTREAM):
    for service in services:
        service_id = service["id"]
        root = upstream / service_id
        source = json.loads((root / "manifest.json").read_text(encoding="utf-8"))
        if source["version"] != service["version"]:
            raise ValueError(f"{service_id}: documentation inventory does not match the catalog version")
        for key in ("publisher", "provenance", "license", "coverage", "omissions"):
            if not source[key].strip():
                raise ValueError(f"{service_id}: missing documentation {key}")
        documents = source["documents"]
        archive = source.get("html_archive")
        if archive:
            documents += archive_documents(root, archive)
        else:
            for page in documents:
                source_path(root, page["path"])
        for notice in source["notices"]:
            source_path(root, notice)
        ids = set()
        for page in documents:
            article_id = page["id"]
            url = urlsplit(page["source_url"])
            if (not DOCUMENT_ID.fullmatch(article_id) or article_id in ids
                    or url.scheme != "https" or not url.hostname or url.username is not None):
                raise ValueError(f"{service_id}: invalid or duplicate publisher article {article_id}")
            ids.add(article_id)
        if ids:
            if source["entry"] not in ids or not source["notices"]:
                raise ValueError(f"{service_id}: offline articles require an entry point and license notices")
        elif source["entry"] is not None:
            raise ValueError(f"{service_id}: entry point without offline articles")
        yield service, root, source
