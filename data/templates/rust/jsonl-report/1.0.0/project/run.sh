#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then set -- examples/requests.jsonl; fi
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-cargo}}" run --frozen -- "$@"
