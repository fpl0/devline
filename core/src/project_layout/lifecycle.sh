#!/bin/sh
set -eu

project=$PWD

usage() {
    cat <<'EOF'
Usage: ./devline.sh COMMAND [ARGUMENTS]

  test                     Run the starter's offline tests
  run | start | stop       Run or manage the app
  status | logs            Inspect the managed app
  prepare [--offline]      Prepare dependencies, documentation and services
  check                    Check offline readiness
  exec COMMAND [ARGS...]   Run a command with prepared offline dependencies
  services ACTION          Manage local services: up, down, status, logs

Starter-specific commands (when provided): build, setup, bundle, smoke, test-services.
App, readiness and service commands use the Devline CLI.
EOF
}

with_devline() {
    cli=${DEVLINE_BIN:-devline}
    if [ -z "${DEVLINE_BIN:-}" ] && ! command -v "$cli" >/dev/null 2>&1 &&
        [ -n "${HOME:-}" ] && [ -x "$HOME/.local/bin/devline" ]; then
        cli="$HOME/.local/bin/devline"
    fi
    if ! command -v "$cli" >/dev/null 2>&1; then
        printf '%s\n' 'Install the Devline CLI or set DEVLINE_BIN to its executable.' >&2
        exit 127
    fi
    exec "$cli" "$@"
}

starter() {
    helper=$1
    shift
    script=".devline/scripts/$helper"
    if [ ! -f "$script" ]; then script="./$helper"; fi
    if [ ! -f "$script" ]; then
        printf 'This starter does not provide %s.\n' "$action" >&2
        exit 2
    fi
    . ./.devline/scripts/toolchain.sh
    exec /bin/sh "$script" "$@"
}

action=${1:-help}
if [ "$#" -gt 0 ]; then shift; fi
case "$action" in
    help | -h | --help) usage ;;
    test)
        if [ -f mix.exs ] && [ -d .devline/cache/elixir/tools ]; then
            # Added clients live in the managed Mix cache. Published starter
            # scripts can reset Mix paths to their original bundled packages.
            . ./.devline/scripts/toolchain.sh
            export MIX_ENV=test ERL_FLAGS="${ERL_FLAGS:-+S 2:2}"
            with_devline --offline project --path "$project" run -- mix test "$@"
        fi
        starter test-offline.sh "$@" ;;
    build) starter build-app.sh "$@" ;;
    setup) starter setup-offline.sh "$@" ;;
    bundle) starter bundle-offline.sh "$@" ;;
    smoke) starter smoke-infra.sh "$@" ;;
    test-services)
        script=.devline/scripts/test-services.py
        if [ ! -f "$script" ]; then script=scripts/test-services.py; fi
        if [ ! -f "$script" ]; then
            printf '%s\n' 'This starter does not provide test-services.' >&2
            exit 2
        fi
        exec python3 "$script" "$@" ;;
    run | start | stop | logs)
        with_devline --offline project --path "$project" "$action" "$@" ;;
    status) with_devline --offline project --path "$project" app-status "$@" ;;
    prepare) with_devline project --path "$project" prepare "$@" ;;
    check) with_devline --offline toolbox check --path "$project" "$@" ;;
    exec)
        if [ "$#" -eq 0 ]; then
            printf '%s\n' 'Usage: ./devline.sh exec COMMAND [ARGUMENTS]' >&2
            exit 2
        fi
        with_devline --offline project --path "$project" run -- "$@" ;;
    services)
        service_action=${1:-}
        if [ "$#" -gt 0 ]; then shift; fi
        case "$service_action" in
            up | down | logs) with_devline --offline infra "$service_action" "$project" "$@" ;;
            status) with_devline --offline infra status --project "$project" "$@" ;;
            *) printf '%s\n' 'Usage: ./devline.sh services <up|down|status|logs>' >&2; exit 2 ;;
        esac ;;
    *) printf 'Unknown Devline command: %s\n' "$action" >&2; usage >&2; exit 2 ;;
esac
