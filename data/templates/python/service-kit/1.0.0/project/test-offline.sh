#!/bin/sh
set -eu
cd "$(dirname "$0")"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-python-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
./setup-offline.sh "$scratch/venv"
PYTEST_DISABLE_PLUGIN_AUTOLOAD=1 "$scratch/venv/bin/python" -B -m pytest -q -p no:cacheprovider
