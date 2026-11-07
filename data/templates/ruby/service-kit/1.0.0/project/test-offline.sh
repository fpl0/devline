#!/bin/sh
set -eu
cd "$(dirname "$0")"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-ruby-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export DEVLINE_GEM_PATH="$scratch/gems" DEVLINE_BUNDLER_HOME="$scratch/bundler"
./bundle-offline.sh install
./bundle-offline.sh exec ruby test_app.rb
