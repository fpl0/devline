#!/bin/sh
set -eu
cd "${1:?project folder}"
export GOWORK=off GOTOOLCHAIN=local CGO_ENABLED=0
"${DEVKIT_TOOLCHAIN:-go}" mod tidy
