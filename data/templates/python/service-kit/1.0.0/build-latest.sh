#!/bin/sh
# Online refresh. Devline tests the result offline before installing a new release.
set -eu
project=${1:?project folder}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
tool=${DEVKIT_TOOLCHAIN:-python3}
"$tool" -m venv "$scratch/builder"
python="$scratch/builder/bin/python"
"$python" -m pip --isolated --disable-pip-version-check --no-cache-dir download \
  --only-binary=:all: --dest "$scratch/wheels" -r "$project/requirements.in"
"$python" - "$scratch/wheels" > "$scratch/requirements.txt" <<'PYTHON'
from email.parser import BytesParser
from hashlib import file_digest
from pathlib import Path
from sys import argv
from zipfile import ZipFile

for wheel in sorted(Path(argv[1]).glob("*.whl")):
    with ZipFile(wheel) as archive:
        metadata = next(name for name in archive.namelist() if name.endswith(".dist-info/METADATA"))
        message = BytesParser().parsebytes(archive.read(metadata), headersonly=True)
    with wheel.open("rb") as stream:
        digest = file_digest(stream, "sha256").hexdigest()
    print(f"{message['Name']}=={message['Version']} --hash=sha256:{digest}")
PYTHON
rm -rf "$project/wheelhouse"
mv "$scratch/wheels" "$project/wheelhouse"
mv "$scratch/requirements.txt" "$project/requirements.txt"
