#!/bin/sh
set -eu
url=${DEVLINE_APP_URL:-http://127.0.0.1:7440}
exec python3 -m uvicorn app:app --host 127.0.0.1 --port "${url##*:}"
