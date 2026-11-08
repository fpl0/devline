#!/bin/sh
set -eu
cd "$(dirname "$0")"
test_script=./test-offline.sh
if [ -f ./.devline/scripts/test-offline.sh ]; then test_script=./.devline/scripts/test-offline.sh; fi
exec /bin/sh "$test_script" "$@"
