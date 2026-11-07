#!/bin/sh
set -eu
cd "$(dirname "$0")"
app=$(./build-app.sh)
exec /usr/bin/open "$app"
