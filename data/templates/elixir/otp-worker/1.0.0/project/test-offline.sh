#!/bin/sh
set -eu
cd "$(dirname "$0")"
export ERL_FLAGS="${ERL_FLAGS:-+S 2:2}"
export HEX_OFFLINE=1 MIX_OS_CONCURRENCY_LOCK=0
scratch=$(mktemp -d "${TMPDIR:-/tmp}/report-queue-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export REPORT_QUEUE_DATABASE="$scratch/queue.json"
"${DEVKIT_TOOLCHAIN:-elixir}" -S mix test
