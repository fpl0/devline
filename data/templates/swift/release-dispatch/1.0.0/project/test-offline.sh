#!/bin/sh
set -eu
cd "$(dirname "$0")"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-dispatch.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
export DEVLINE_SWIFT_SCRATCH="$scratch/swift"
./scripts/swift-local.sh run ReleaseDispatch --self-test
