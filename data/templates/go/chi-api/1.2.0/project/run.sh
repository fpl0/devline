#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ -n "${DEVLINE_APP_URL:-}" ]; then
    export LISTEN_ADDR="${DEVLINE_APP_URL#http://}"
fi
export GOWORK=off GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off CGO_ENABLED=0
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-go}}" run -mod=vendor . "$@"
