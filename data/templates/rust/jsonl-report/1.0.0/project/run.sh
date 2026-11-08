#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then set -- --field /http/status examples/requests.jsonl; fi
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-cargo}}" run --frozen -- "$@"
