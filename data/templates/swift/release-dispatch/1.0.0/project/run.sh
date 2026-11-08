#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec ./scripts/swift-local.sh run ReleaseDispatch "$@"
