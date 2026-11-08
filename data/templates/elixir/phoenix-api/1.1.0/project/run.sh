#!/bin/sh
set -eu
cd "$(dirname "$0")"
tools="$PWD/vendor-tools"
[ -d "$tools" ] || tools="$PWD/.devline/packages/vendor-tools"
export PHX_SERVER=1 HEX_OFFLINE=1 EXQLITE_FORCE_BUILD=1 MIX_OS_CONCURRENCY_LOCK=0
export MIX_REBAR3="$tools/rebar3" MIX_ARCHIVES="$tools/archives"
if [ -n "${DEVLINE_POSTGRES_URL:-}" ] && [ -n "${DEVLINE_MYSQL_URL:-}" ] &&
   [ -n "${DEVLINE_RABBITMQ_URL:-}" ] && [ -n "${DEVLINE_REDIS_URL:-}" ] &&
   [ -n "${DEVLINE_SEAWEEDFS_URL:-}" ] && [ -n "${DEVLINE_MEILISEARCH_URL:-}" ] &&
   [ -n "${DEVLINE_MAILPIT_WEB_PORT:-}" ]; then
    export RELEASE_DESK=1
fi
exec "${DEVKIT_TOOLCHAIN:-elixir}" -S mix run --no-halt
