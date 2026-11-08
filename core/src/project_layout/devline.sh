#!/bin/sh
set -eu
project=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd -P)
cd "$project"
exec /bin/sh .devline/scripts/lifecycle.sh "$@"
