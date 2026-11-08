#!/bin/sh
set -eu
cd "$(dirname "$0")"
url=${DEVLINE_APP_URL:-http://127.0.0.1:${1:-3000}}
port=${url##*:}
set -- ruby bin/rails
# Devline's Ruby adapter launches this script inside the prepared bundle.
if [ -z "${BUNDLE_BIN_PATH:-}" ]; then
  ./bundle-offline.sh install
  set -- ./bundle-offline.sh exec "$@"
fi
"$@" db:prepare
exec "$@" server --binding 127.0.0.1 --port "$port"
