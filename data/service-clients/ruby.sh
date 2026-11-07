#!/bin/sh
set -eu
base=$(cd "$(dirname "$0")" && pwd)
exec python3 "$base/apply-extra.py" "ruby" "$@"
