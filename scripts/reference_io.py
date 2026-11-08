"""Read upstream inputs and write document pack shards."""

import json
from pathlib import Path
import shutil
import subprocess
import tarfile
from tempfile import TemporaryDirectory


SHARD_BYTES = 64 * 1024 * 1024


def read_text(file):
    return file.read_text(encoding="utf-8-sig", errors="replace")


def atomic_json(path, value):
    with TemporaryDirectory(prefix=".catalog-", dir=path.parent) as scratch:
        staged = Path(scratch) / path.name
        staged.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        staged.replace(path)


def write_shards(target, documents):
    """Bound source archive files while keeping individual articles intact."""
    number = 1
    total = 0
    shard = []
    size = 4
    for document in documents:
        encoded = json.dumps(document, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        if len(encoded) + 4 > SHARD_BYTES:
            raise RuntimeError(f"Article exceeds shard size: {document['id']}")
        if shard and size + len(encoded) + 2 > SHARD_BYTES:
            total += write_shard(target, number, shard)
            number += 1
            shard = []
            size = 4
        shard.append(encoded)
        size += len(encoded) + 2
    total += write_shard(target, number, shard)
    return number, total


def write_shard(target, number, rows):
    path = target / ("documents.json" if number == 1 else f"documents-{number}.json")
    temporary = path.with_suffix(".tmp")
    with temporary.open("wb") as output:
        output.write(b"[\n")
        for index, row in enumerate(rows):
            if index:
                output.write(b",\n")
            output.write(row)
        output.write(b"\n]\n")
    temporary.replace(path)
    return path.stat().st_size


def safe_extract_tar(path, destination, select):
    with tarfile.open(path) as archive:
        for entry in archive:
            name = Path(entry.name)
            if not entry.isfile() or name.is_absolute() or ".." in name.parts or not select(name):
                continue
            target = destination / name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(entry) as source, target.open("wb") as output:
                shutil.copyfileobj(source, output)


def command(*args, cwd=None, env=None, check=True):
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True, check=False)
    if check and result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-1600:]}")
    return result.stdout if result.returncode == 0 else ""
