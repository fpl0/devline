#!/bin/sh
# Rebuild with the selected local Swift toolchain and SDK. Devline verifies the release next.
set -eu
cd "${1:?project directory}"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-swift-rebuild.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
export DEVLINE_SWIFT_SCRATCH="$scratch"
./scripts/swift-local.sh package dump-package >/dev/null
printf '%s\n' 'Fieldnotes uses the selected macOS SDK and local Swift targets.'
