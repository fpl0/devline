#!/bin/sh
# Online preparation. Offline verification is required before the release is installed.
set -eu
project=${1:?project folder}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

tool=${DEVKIT_TOOLCHAIN:-python3}
"$tool" -m venv "$scratch/builder"
builder="$scratch/builder/bin/python"
"$builder" -m pip --isolated --disable-pip-version-check --no-cache-dir download \
  --only-binary=:all: --dest "$scratch/wheels" 'fastapi<1' 'uvicorn<1' 'httpx<1'
rm -rf "$project/wheelhouse"
mv "$scratch/wheels" "$project/wheelhouse"
"$builder" - "$project" <<'PYTHON'
from email.parser import BytesParser
from hashlib import sha256
from pathlib import Path
from sys import argv
from zipfile import ZipFile

project = Path(argv[1])
requirements = []
for wheel in sorted((project / "wheelhouse").glob("*.whl")):
    with ZipFile(wheel) as archive:
        metadata = next(name for name in archive.namelist() if name.endswith(".dist-info/METADATA"))
        message = BytesParser().parsebytes(archive.read(metadata), headersonly=True)
    requirements.append(f"{message['Name']}=={message['Version']} --hash=sha256:{sha256(wheel.read_bytes()).hexdigest()}")
(project / "requirements.txt").write_text("\n".join(requirements) + "\n")
PYTHON
