#!/usr/bin/env python3
"""Build the runtime catalog from the documentation and starter catalogs."""

import argparse
import json
from pathlib import Path
from tempfile import TemporaryDirectory


def read_releases(path):
    releases = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(releases, list):
        raise ValueError(f"{path.name} must contain an array of releases")
    for index, release in enumerate(releases, start=1):
        if not isinstance(release, dict) or any(
            not isinstance(release.get(key), str) or not release[key]
            for key in ("id", "version", "path")
        ):
            raise ValueError(f"{path.name} release {index} needs an id, version, and path")
    return releases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, default=Path(__file__).resolve().parents[1] / "data")
    parser.add_argument("--check", action="store_true", help="fail if the runtime catalog needs regeneration")
    options = parser.parse_args()

    target = options.data / "catalog.json"
    try:
        catalog = {
            "schema_version": 1,
            "docs": read_releases(options.data / "docs-catalog.json"),
            "templates": read_releases(options.data / "templates-catalog.json"),
        }
        expected = (json.dumps(catalog, indent=2) + "\n").encode("utf-8")
        if target.exists() and target.read_bytes() == expected:
            return
        if options.check:
            raise SystemExit(f"{target} is out of date. Run python3 scripts/sync-catalog.py.")
        with TemporaryDirectory(prefix=".catalog-", dir=target.parent) as scratch:
            staged = Path(scratch) / target.name
            staged.write_bytes(expected)
            staged.replace(target)
    except (OSError, ValueError) as error:
        raise SystemExit(str(error)) from error
    print(f"Updated {target}")


if __name__ == "__main__":
    main()
