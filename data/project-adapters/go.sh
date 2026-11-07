#!/bin/sh
set -eu
action=${1:?action}; project=${2:?project folder}; cache=${3:?managed cache}; tool=${4:?Go executable}
shift 4
cd "$project"
[ -f go.mod ] || { echo 'Open the directory containing go.mod.' >&2; exit 1; }
export GOWORK=off GOTOOLCHAIN=local
export PATH="$(dirname "$tool"):$PATH"
case "$action" in
    prepare)
        mkdir -p "$cache/go"
        GOCACHE="$HOME/go-build" GOMODCACHE="$HOME/go-mod" "$tool" mod vendor
        ;;
    verify)
        export GOPROXY=off GOSUMDB=off GOCACHE="$HOME/go-build" GOMODCACHE="$HOME/go-mod"
        exec "$tool" test -mod=vendor ./...
        ;;
    run)
        export GOPROXY=off GOSUMDB=off GOFLAGS=-mod=vendor
        exec "$@"
        ;;
    *) exit 2 ;;
esac
