#!/bin/sh
set -eu
action=${1:?prepare or verify}
project=${2:?staged project}
cache=${3:?managed dependency directory}/elixir
tool=${4:?Elixir executable}
shift 4
adapters=$(cd "$(dirname "$0")" && pwd)
toolkit="$adapters/../templates/elixir/phoenix-api/1.0.0/project/vendor-tools"
project=$(cd "$project" && pwd -P)
cache=$(python3 -c 'from pathlib import Path;import sys;print(Path(sys.argv[1]).resolve())' "$cache")
export TMPDIR="$(cd "${TMPDIR:?}" && pwd -P)/"
cd "$project"
if [ ! -f mix.exs ]; then
    printf '%s\n' 'Choose the Mix project directory containing mix.exs.' >&2
    exit 1
fi
export PATH="$(dirname "$tool"):$PATH"
export DEVKIT_TOOLCHAIN="$tool"
if [ "$action" = run ]; then
    [ "$#" -gt 0 ] || { printf '%s\n' 'Specify a Mix project command.' >&2; exit 2; }
    if [ ! -d "$cache/tools" ]; then
        if [ -f .devkit-origin.json ] && [ -x ./test-offline.sh ]; then
            export HEX_OFFLINE=1 EXQLITE_FORCE_BUILD=1 MIX_OS_CONCURRENCY_LOCK=0
            export MIX_HOME="$project/_build/devline-home" HEX_HOME="$project/_build/devline-hex"
            if [ -d "$project/vendor-tools" ]; then
                export MIX_REBAR3="$project/vendor-tools/rebar3" MIX_ARCHIVES="$project/vendor-tools/archives"
            fi
            exec "$@"
        fi
        printf '%s\n' 'Prepare this project before running with local Mix dependencies.' >&2
        exit 1
    fi
    scratch=$(mktemp -d)
    trap 'rm -rf "$scratch"' EXIT
    trap 'exit 130' HUP INT TERM
    /bin/cp -cR "$cache" "$scratch/cache"
    cache="$scratch/cache"
    export MIX_ENV="${MIX_ENV:-test}" MIX_BUILD_PATH="$project/_build/devline-${MIX_ENV:-test}"
elif [ "$action" = verify ] && [ -d "$cache/tools" ]; then
    /bin/cp -cR "$cache" "${TMPDIR:?}/mix-cache"
    cache="$TMPDIR/mix-cache"
fi
export MIX_HOME="$cache/mix" HEX_HOME="$cache/hex" MIX_DEPS_PATH="$cache/deps"
export MIX_ARCHIVES="$cache/tools/archives" MIX_REBAR3="$cache/tools/rebar3"
export MIX_BUILD_PATH="${MIX_BUILD_PATH:-${TMPDIR:?}/mix-build}" MIX_ENV="${MIX_ENV:-test}" MIX_OS_CONCURRENCY_LOCK=0
export EXQLITE_FORCE_BUILD=1

case "$action" in
    prepare)
        mkdir -p "$cache"
        if [ ! -d "$cache/tools" ]; then cp -R "$toolkit" "$cache/tools"; fi
        before=$(mktemp "$TMPDIR/mix-lock.XXXXXX")
        trap 'rm -f "$before"' EXIT
        trap 'exit 130' HUP INT TERM
        if [ -f mix.lock ]; then cp mix.lock "$before"; fi
        HEX_OFFLINE=${DEVLINE_OFFLINE:-0} "$tool" -S mix deps.get
        if [ -f mix.lock ] && ! cmp -s "$before" mix.lock; then
            printf '%s\n' 'Resolve and save the intended Mix dependency versions in mix.lock, then prepare again.' >&2
            exit 1
        fi
        ;;
    verify)
        if [ ! -x "$MIX_REBAR3" ] || [ ! -d "$MIX_ARCHIVES" ]; then
            if [ -f .devkit-origin.json ] && [ -x ./test-offline.sh ]; then exec ./test-offline.sh; fi
            printf '%s\n' 'The prepared local Hex and Rebar tools are missing.' >&2
            exit 1
        fi
        export HEX_OFFLINE=1
        "$tool" -S mix test
        ;;
    run)
        export HEX_OFFLINE=1
        "$@"
        ;;
    *) printf '%s\n' 'Expected prepare or verify.' >&2; exit 2 ;;
esac
