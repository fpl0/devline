#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ -n "${DEVKIT_TOOLCHAIN:-}" ]; then
  export PATH="$(dirname "$DEVKIT_TOOLCHAIN"):$PATH"
fi
export MIX_OS_CONCURRENCY_LOCK=0 HEX_OFFLINE=1 EXQLITE_FORCE_BUILD=1 MIX_REBAR3="$PWD/vendor-tools/rebar3" MIX_ARCHIVES="$PWD/vendor-tools/archives"
export ERL_FLAGS="${ERL_FLAGS:-+S 2:2}"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/incident-room-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export INCIDENT_DATABASE="$scratch/incidents.json"
mix test
