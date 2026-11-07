#!/bin/sh
set -eu
cd "$(dirname "$0")"
export GOWORK=off GOTOOLCHAIN=local GOPROXY=off GOSUMDB=off CGO_ENABLED=0
exec "${DEVKIT_TOOLCHAIN:-go}" test -mod=vendor -tags=integration -count=1 -run '^TestLiveInfrastructure$' ./...
