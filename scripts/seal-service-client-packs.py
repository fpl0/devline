#!/usr/bin/env python3
"""Seal prepared client packs after their offline proof, or validate existing seals."""

import argparse
import hashlib
import json
from pathlib import Path
from stat import S_ISDIR, S_ISREG
from tempfile import TemporaryDirectory


DATA = Path(__file__).resolve().parents[1] / "data"
METADATA_FILES = {"inventory.json", "ready.json"}


def pack_inventory(root):
    entries = []
    for file in sorted(root.rglob("*")):
        metadata = file.lstat()
        if S_ISDIR(metadata.st_mode):
            continue
        if not S_ISREG(metadata.st_mode):
            raise ValueError(f"Client pack contains a symbolic link or special file: {file}")
        relative = file.relative_to(root).as_posix()
        if relative in METADATA_FILES:
            continue
        with file.open("rb") as stream:
            checksum = hashlib.file_digest(stream, "sha256").hexdigest()
        entries.append({"path": relative, "bytes": metadata.st_size, "sha256": checksum})
    return entries


def seal_pack(root, *, check, data):
    ready = root / "ready.json"
    inventory = root / "inventory.json"
    if root.is_symlink() or ready.is_symlink() or inventory.is_symlink():
        raise ValueError(f"Client pack paths must not contain symbolic links: {root}")
    if not ready.is_file():
        raise ValueError(f"No completed proof marker: {root}")
    metadata = json.loads(ready.read_text(encoding="utf-8"))
    if (not isinstance(metadata, dict) or type(metadata.get("schema_version")) is not int
            or metadata["schema_version"] != 1 or metadata.get("stack") != root.name):
        raise ValueError(f"Invalid client pack identity: {ready}")
    entries = pack_inventory(root)
    content = (json.dumps({"schema_version": 1, "stack": root.name, "files": entries}, indent=2) + "\n").encode("utf-8")
    digest = hashlib.sha256(content).hexdigest()
    if check:
        if not inventory.is_file() or inventory.read_bytes() != content or metadata.get("inventory_sha256") != digest:
            raise ValueError(f"{root.name} seal changed")
    else:
        metadata["inventory_sha256"] = digest
        with TemporaryDirectory(prefix=".client-seal-", dir=data) as scratch:
            staged = Path(scratch)
            (staged / inventory.name).write_bytes(content)
            (staged / ready.name).write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
            (staged / inventory.name).replace(inventory)
            (staged / ready.name).replace(ready)
    print(f"{'Verified' if check else 'Sealed'} {root.name}: {len(entries)} files")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, default=DATA)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("stacks", nargs="*")
    options = parser.parse_args()

    clients = options.data / "service-clients"
    artifacts = clients / "artifacts"
    try:
        if clients.is_symlink() or artifacts.is_symlink():
            raise ValueError(f"Client artifact directory must not be a symbolic link: {artifacts}")
        present = {path.name for path in artifacts.iterdir() if path.is_dir() or path.is_symlink()}
        expected = {path.stem for path in clients.glob("*.json")}
        stacks = present | expected
        if not stacks:
            raise ValueError(f"No service client packs found: {artifacts}")
        unknown = set(options.stacks) - stacks
        if unknown:
            raise ValueError(f"Unknown client pack stacks: {', '.join(sorted(unknown))}")
        for stack in sorted(set(options.stacks) or stacks):
            seal_pack(artifacts / stack, check=options.check, data=options.data)
    except (OSError, ValueError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
