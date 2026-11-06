#!/bin/sh
set -eu
cd "$(dirname "$0")"
python="${DEVKIT_TOOLCHAIN:-python3}"
"$python" -c 'import platform, sys; sys.exit(0 if sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 15) and platform.system() == "Darwin" and platform.machine() == "arm64" else "Requires Apple Silicon macOS and CPython 3.15")'
"$python" -m venv .venv
.venv/bin/python -E -s -m pip --isolated --disable-pip-version-check --no-cache-dir install --no-index --find-links=wheelhouse --require-hashes -r requirements.txt
exec .venv/bin/python -E -s smoke.py
