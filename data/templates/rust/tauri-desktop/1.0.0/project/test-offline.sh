#!/bin/sh
set -eu
cd "$(dirname "$0")"
export MACOSX_DEPLOYMENT_TARGET=14.0
exec "${DEVKIT_TOOLCHAIN:-cargo}" test --frozen --jobs "${CARGO_BUILD_JOBS:-2}"
