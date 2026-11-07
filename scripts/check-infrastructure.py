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


DIGEST = re.compile(r"sha256:[0-9a-f]{64}")
PLACEHOLDER = re.compile(r"\{\{(port|secret):([a-z][a-z0-9-]*)\}\}")


def check_catalog(data):
    catalog = json.loads((data / "infrastructure.json").read_text())
    if catalog["schema_version"] != 1:
        raise ValueError("Unsupported infrastructure catalog schema")
    packs = json.loads((data / "docs-catalog.json").read_text())
    doc_refs = set()
    for pack in packs:
        if pack["id"] != "infrastructure":
            continue
        for record in json.loads((data / pack["path"] / "documents.json").read_text()):
            source = urlsplit(record["source_url"])
            publisher = (
                record["id"].startswith("reference/postgresql/")
                and source.hostname == "www.postgresql.org"
                and source.path.startswith("/docs/18/")
            ) or (
                record["id"].startswith("reference/seaweedfs/")
                and source.hostname == "github.com"
                and source.path.startswith("/seaweedfs/seaweedfs/blob/")
            )
            if not publisher or source.scheme != "https" or record["category"] != "reference":
                raise ValueError(f"Non-publisher infrastructure reference: {record['id']}")
            doc_refs.add(f"{pack['id']}@{pack['version']}/{record['id']}")
    releases = set()
    for service in catalog["services"]:
        name = f"{service['id']}@{service['version']}"
        image = service["image"]
        if name in releases:
            raise ValueError(f"{name}: duplicate release")
        reference = service.get("docs_ref")
        if reference is not None and reference not in doc_refs:
            raise ValueError(f"{name}: missing official offline reference")
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
