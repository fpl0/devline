#!/bin/sh
set -eu
cd "$(dirname "$0")"
url=${DEVLINE_APP_URL:-http://127.0.0.1:7480}
port=${url##*:}
host=${url%:*}
host=${host#http://}
host=${host#\[}
host=${host%\]}
exec "${DEVKIT_TOOLCHAIN:-node}" node_modules/vite/bin/vite.js --host "$host" --port "$port" --strictPort
