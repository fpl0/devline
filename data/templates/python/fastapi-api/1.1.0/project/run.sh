#!/bin/sh
set -eu
cd "$(dirname "$0")"
python=${DEVKIT_TOOLCHAIN:-python3}
if ! "$python" -c 'import uvicorn' 2>/dev/null; then
  [ -x .venv/bin/python ] || ./setup-offline.sh
  python=.venv/bin/python
fi
url=${DEVLINE_APP_URL:-http://127.0.0.1:7441}
exec "$python" -m uvicorn app:app --host 127.0.0.1 --port "${url##*:}"
