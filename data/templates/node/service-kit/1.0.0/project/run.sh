#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-node}}" src/server.mjs
