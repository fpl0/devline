#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then set -- snapshot --root examples --manifest .data/example-inventory.json; fi
export GOWORK=off GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off CGO_ENABLED=0
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-go}}" run -p=2 -mod=vendor . "$@"
