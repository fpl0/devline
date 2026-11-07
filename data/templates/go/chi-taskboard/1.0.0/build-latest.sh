#!/bin/sh
# Online preparation. Offline verification is required before the release is installed.
set -eu
project=${1:?project folder}
scratch=$(mktemp -d)
cleanup() {
    status=$?
    chmod -R u+w "$scratch" 2>/dev/null || :
    rm -rf "$scratch" || :
    exit "$status"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

tool=${DEVKIT_TOOLCHAIN:-go}
cd "$project"
export GOWORK=off GOTOOLCHAIN=local CGO_ENABLED=0
export GOMODCACHE="$scratch/resolve-cache" GOCACHE="$scratch/build-cache"
"$tool" get github.com/go-chi/chi/v5@latest github.com/jackc/pgx/v5@latest github.com/redis/go-redis/v9@latest
"$tool" mod tidy
"$tool" mod vendor
