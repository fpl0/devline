#!/bin/sh
set -eu
cd "$(dirname "$0")"
export HEX_OFFLINE=1 MIX_OS_CONCURRENCY_LOCK=0
exec "${DEVKIT_TOOLCHAIN:-elixir}" -S mix test
