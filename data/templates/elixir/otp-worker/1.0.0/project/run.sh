#!/bin/sh
set -eu
cd "$(dirname "$0")"
export ERL_FLAGS="${ERL_FLAGS:-+S 2:2}" HEX_OFFLINE=1 MIX_OS_CONCURRENCY_LOCK=0
if [ "$#" -gt 0 ]; then exec "${DEVKIT_TOOLCHAIN:-elixir}" -S mix run queue.exs "$@"; fi
exec "${DEVKIT_TOOLCHAIN:-elixir}" -S mix run demo.exs
