#!/bin/sh
set -eu
cd "$(dirname "$0")"
app=$(./build-app.sh)
if [ "${1:-}" = "--managed" ]; then
    exec "$app/Contents/MacOS/IdeaBoard"
fi
exec /usr/bin/open "$app"
