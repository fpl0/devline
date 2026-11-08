#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ -n "${DEVLINE_APP_URL:-}" ]; then
    export AXUM_LISTEN="${DEVLINE_APP_URL#http://}"
fi
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-cargo}}" run --frozen -- "$@"
