#!/bin/sh
set -eu
cd "$(dirname "$0")"
python=${DEVKIT_TOOLCHAIN:-python3}
if [ "$#" -gt 0 ]; then exec "$python" -m fieldnotes "$@"; fi
"$python" -m fieldnotes index examples/notes
exec "$python" -m fieldnotes search 'reliable delivery'
