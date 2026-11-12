#!/bin/sh
set -eu
cd "$(dirname "$0")"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/order-ledger-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
./setup-offline.sh "$scratch/venv"
export LEDGER_DATABASE="$scratch/import.sqlite3"
"$scratch/venv/bin/python" -E -s -B smoke.py
