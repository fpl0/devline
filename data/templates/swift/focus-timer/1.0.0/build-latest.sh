#!/bin/sh
set -eu
cd "${1:?project directory}"
./scripts/swift-local.sh package dump-package >/dev/null
printf '%s\n' 'Focus Sessions uses local Swift targets and the selected macOS SDK.'
