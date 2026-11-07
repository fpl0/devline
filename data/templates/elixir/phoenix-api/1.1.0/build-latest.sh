#!/bin/sh
set -eu
project=${1:?project folder}
tool=${DEVKIT_TOOLCHAIN:-elixir}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export MIX_HOME="$scratch/mix" HEX_HOME="$scratch/hex" MIX_DEPS_PATH="$scratch/deps" MIX_BUILD_PATH="$scratch/build"
export MIX_ARCHIVES="$project/vendor-tools/archives" MIX_OS_CONCURRENCY_LOCK=0 MIX_ENV=test HEX_OFFLINE=0
cd "$project"
mv vendor "$scratch/old-vendor"
"$tool" -S mix deps.get
python3 - "$project" "$HEX_HOME/packages/hexpm" <<'PYTHON'
from pathlib import Path
import hashlib
import io
import re
import sys
import tarfile

project = Path(sys.argv[1])
cache = Path(sys.argv[2])
lock = (project / "mix.lock").read_text()
vendor = project / "vendor"
vendor.mkdir()
dependencies = []
for name, version, checksum in re.findall(r'"([\w]+)": \{:hex, :\w+, "([^"]+)", .+, "hexpm", "([0-9a-f]+)"\}', lock):
    data = (cache / f"{name}-{version}.tar").read_bytes()
    if hashlib.sha256(data).hexdigest() != checksum:
        raise ValueError(f"Hex package hash mismatch: {name}")
    destination = vendor / name
    destination.mkdir()
    with tarfile.open(fileobj=io.BytesIO(data)) as outer:
        contents = outer.extractfile("contents.tar.gz").read()
        with tarfile.open(fileobj=io.BytesIO(contents), mode="r:gz") as inner:
            inner.extractall(destination, filter="data")
    manager = ""
    if name == "ranch":
        # The Hex package includes a complete app descriptor. Rebar compiles
        # it from local source without erlang.mk's whitespace-sensitive paths.
        app = destination / "src/ranch.app.src"
        if not app.exists():
            app.write_bytes((destination / "ebin/ranch.app").read_bytes())
        config = destination / "rebar.config"
        if not config.exists():
            config.write_text("{erl_opts, [debug_info]}.\n")
        manager = ", manager: :rebar3"
    dependencies.append(f'  {{:{name}, path: "vendor/{name}", override: true{manager}}},')
if not dependencies:
    raise ValueError("No resolved Hex dependencies")
(vendor / "dependencies.exs").write_text("[\n" + "\n".join(dependencies) + "\n]\n")
(project / "mix.lock").rename(vendor / "upstream-lock.exs")
PYTHON
