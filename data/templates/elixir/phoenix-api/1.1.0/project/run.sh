#!/bin/sh
set -eu
cd "$(dirname "$0")"
tools="$PWD/vendor-tools"
[ -d "$tools" ] || tools="$PWD/.devline/packages/vendor-tools"
export PHX_SERVER=1 HEX_OFFLINE=1 EXQLITE_FORCE_BUILD=1 MIX_OS_CONCURRENCY_LOCK=0
export MIX_REBAR3="$tools/rebar3" MIX_ARCHIVES="$tools/archives"
export ERL_FLAGS="${ERL_FLAGS:-+S 2:2}"

exec "${DEVKIT_TOOLCHAIN:-elixir}" -S mix run --no-halt
