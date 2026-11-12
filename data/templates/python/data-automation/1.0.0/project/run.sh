#!/bin/sh
set -eu
cd "$(dirname "$0")"
python=${DEVKIT_TOOLCHAIN:-python3}
if ! "$python" -c 'import duckdb' 2>/dev/null; then
  [ -x .venv/bin/python ] || ./setup-offline.sh
  python=.venv/bin/python
fi
if [ "$#" -gt 0 ]; then exec "$python" etl.py "$@"; fi
"$python" etl.py ingest fixtures/sales.csv
exec "$python" etl.py report --html data/report.html
