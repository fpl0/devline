#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec /bin/sh scripts/with-browser.sh src/cli.mjs "$@"
