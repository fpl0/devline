#!/usr/bin/env python3
"""Check service catalog wiring and optionally verify exact public image pins."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import urlsplit

from infrastructure_sources import REFERENCE, document_id, read_sources, source_path


DIGEST = re.compile(r"sha256:[0-9a-f]{64}")
PLACEHOLDER = re.compile(r"\{\{(port|secret):([a-z][a-z0-9-]*)\}\}")


def check_references(data, services):
    packs = json.loads((data / "docs-catalog.json").read_text())
    installed_catalog = json.loads((data / "catalog.json").read_text())
    if installed_catalog["docs"] != packs:
        raise ValueError("The runtime documentation catalog differs from the source catalog")
    pack, = [item for item in packs if f"{item['id']}@{item['version']}" == REFERENCE]
    root = data / pack["path"]
    sources = {}
    for index in range(1, pack.get("shards", 1) + 1):
        name = "documents.json" if index == 1 else f"documents-{index}.json"
        for record in json.loads((root / name).read_text()):
            if record["category"] != "reference" or record["id"] in sources:
                raise ValueError(f"Non-reference or duplicate infrastructure article: {record['id']}")
            sources[record["id"]] = record["source_url"]
    expected = {}
    scope = {"coverage": [], "omissions": []}
    for service, directory, source in read_sources(services):
        service_id = service["id"]
        entry = f"{REFERENCE}/{document_id(service_id, source['entry'])}" if source["entry"] else None
        if service.get("docs_ref") != entry:
            raise ValueError(f"{service_id}: wrong official offline entry point")
        for field in ("coverage", "omissions"):
            if service.get(f"docs_{field}") != source[field]:
                raise ValueError(f"{service_id}: missing or stale documentation {field}")
            scope[field].append(f"{service['name']} {service['version']} — {source[field]}")
        for page in source["documents"]:
            expected[document_id(service_id, page["id"])] = page["source_url"]
        for notice in source["notices"]:
            included = root / f"{service_id.upper()}-{Path(notice).name}"
            if included.read_text(encoding="utf-8") != source_path(directory, notice).read_text(encoding="utf-8"):
                raise ValueError(f"{service_id}: publisher notice differs from its source")
    if any(pack[field] != entries for field, entries in scope.items()):
        raise ValueError("The infrastructure docset scope differs from its source inventory")
    missing = expected.keys() - sources.keys()
    unexpected = sources.keys() - expected.keys()
    wrong_source = {key for key in expected.keys() & sources.keys() if sources[key] != expected[key]}
    if missing or unexpected or wrong_source:
        raise ValueError(f"Publisher article inventory mismatch. Missing: {sorted(missing)}. "
                         f"Unexpected: {sorted(unexpected)}. Wrong source: {sorted(wrong_source)}.")
    print(f"Verified {len(sources)} publisher reference articles in {REFERENCE}")


def check_catalog(data):
    catalog = json.loads((data / "infrastructure.json").read_text())
    if catalog["schema_version"] != 1:
        raise ValueError("Unsupported infrastructure catalog schema")
    check_references(data, catalog["services"])
    releases = set()
    for service in catalog["services"]:
        name = f"{service['id']}@{service['version']}"
        image = service["image"]
        if name in releases:
            raise ValueError(f"{name}: duplicate release")
        help_ref = service.get("help_ref", "")
        if not re.fullmatch(r"[a-z][a-z0-9-]*", help_ref):
            raise ValueError(f"{name}: invalid Devline help ID")
        guide = data / "help" / "infrastructure" / f"{help_ref}.md"
        if not guide.is_file() or not guide.read_text().startswith("# "):
            raise ValueError(f"{name}: missing Devline help page")
        releases.add(name)
        if (image["platform"] != "linux/arm64" or not image["compressed_bytes"] > 0
                or not all(DIGEST.fullmatch(image[key]) for key in ("index_digest", "digest", "config_digest"))):
            raise ValueError(f"{name}: missing arm64 image pins")
        sources = service["provenance"]
        for key in ("image_url", "source_url", "docs_url", "license_url"):
            url = urlsplit(sources[key])
            if url.scheme != "https" or not url.hostname or url.username is not None:
                raise ValueError(f"{name}: expected an HTTPS source URL")
        ports = {port["id"] for port in service["ports"]}
        if len(ports) != len(service["ports"]) or any(
            not 1 <= port[key] <= 65535
            for port in service["ports"] for key in ("host_port", "container_port")
        ):
            raise ValueError(f"{name}: invalid port mapping")
        targets = [volume["target"] for volume in service["volumes"]]
        if len(set(targets)) != len(targets) or any(not path.startswith("/") for path in targets):
            raise ValueError(f"{name}: invalid data volume")
        health = service["healthcheck"]["command"]
        if len(health) < 2 or health[0] not in {"CMD", "CMD-SHELL"}:
            raise ValueError(f"{name}: invalid health check")
        if service["id"] == "keycloak" and service["connection"] != "http://127.0.0.1:{{port:http}}":
            raise ValueError(f"{name}: advertise the running server address; application realms and clients are configured explicitly")
        credentials = {credential["id"] for credential in service["credentials"]}
        values = [service["connection"], *service["env"].values(), *service["command"], *health]
        for value in values:
            for kind, key in PLACEHOLDER.findall(value):
                if key not in (ports if kind == "port" else credentials):
                    raise ValueError(f"{name}: unknown {kind} placeholder {key}")
            if "{{" in PLACEHOLDER.sub("", value) or "}}" in PLACEHOLDER.sub("", value):
                raise ValueError(f"{name}: malformed placeholder")
    return catalog["services"]


def manifest(reference, expected_digest):
    raw = subprocess.check_output(
        ["docker", "buildx", "imagetools", "inspect", "--raw", reference], timeout=45)
    # Buildx versions differ in whether they append a presentation newline.
    if all("sha256:" + hashlib.sha256(value).hexdigest() != expected_digest
           for value in (raw, raw.removesuffix(b"\n"))):
        raise ValueError(f"Registry manifest digest changed for {reference}")
    return json.loads(raw)


def check_registry(service):
    image = service["image"]
    base = f"{image['registry']}/{image['repository']}"
    index = manifest(f"{base}:{image['tag']}", image["index_digest"])
    matching = [item["digest"] for item in index["manifests"]
                if item.get("platform", {}).get("os") == "linux"
                and item["platform"].get("architecture") == "arm64"]
    if matching != [image["digest"]]:
        raise ValueError(f"Platform manifest changed for {base}:{image['tag']}")
    selected = manifest(f"{base}@{image['digest']}", image["digest"])
    if (selected["config"]["digest"] != image["config_digest"]
            or sum(layer["size"] for layer in selected["layers"]) != image["compressed_bytes"]):
        raise ValueError(f"Image configuration or size changed for {base}:{image['tag']}")
    print(f"Verified {service['id']} {image['tag']} {image['platform']}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, default=Path(__file__).resolve().parents[1] / "data")
    parser.add_argument("--registry", action="store_true", help="read registry manifests using Docker Buildx without pulling images")
    options = parser.parse_args()
    services = check_catalog(options.data)
    if options.registry:
        with ThreadPoolExecutor(max_workers=5) as pool:
            list(pool.map(check_registry, services))
    print(f"Verified service catalog, separate Devline help, and official reference links for {len(services)} releases")


if __name__ == "__main__":
    main()
