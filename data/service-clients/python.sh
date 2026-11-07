#!/bin/sh
set -eu
base=$(cd "$(dirname "$0")" && pwd)
exec "$4" -I "$base/python.py" "$@"
