#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ -z "${BUNDLE_BIN_PATH:-}" ]; then
  ./bundle-offline.sh install
  exec ./bundle-offline.sh exec /bin/sh "$0" "$@"
fi
if [ "$#" -gt 0 ]; then exec ruby automation.rb "$@"; fi
scratch=$(mktemp -d)
scratch=$(CDPATH='' cd "$scratch" && pwd -P)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
ruby automation.rb capture fixtures/workspace --name example --vault "$scratch/vault"
ruby automation.rb verify example --vault "$scratch/vault"
ruby automation.rb restore example "$scratch/restored" --vault "$scratch/vault"
printf 'Verified a complete snapshot and restore. Use ./run.sh help for your own files.\n'
