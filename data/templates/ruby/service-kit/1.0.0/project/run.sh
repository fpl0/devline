#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ -z "${BUNDLE_BIN_PATH:-}" ]; then
  ./bundle-offline.sh install
  exec ./bundle-offline.sh exec /bin/sh "$0" "$@"
fi
url=${DEVLINE_APP_URL:-http://127.0.0.1:7461}
exec ruby -S rackup -s webrick -o 127.0.0.1 -p "${url##*:}"
