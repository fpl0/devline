#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ -n "${DEVKIT_TOOLCHAIN:-}" ]; then
  export PATH="$(dirname "$DEVKIT_TOOLCHAIN"):$PATH"
fi
export MIX_OS_CONCURRENCY_LOCK=0 HEX_OFFLINE=1 EXQLITE_FORCE_BUILD=1 MIX_REBAR3="$PWD/vendor-tools/rebar3" MIX_ARCHIVES="$PWD/vendor-tools/archives"
exec mix test
