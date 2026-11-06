#!/bin/sh
# Online preparation. Offline verification is required before the release is installed.
set -eu
project=${1:?project folder}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

tool=${DEVKIT_TOOLCHAIN:-cargo}
cd "$project"
mv .cargo/config.toml "$scratch/previous-config"
mv vendor "$scratch/previous-vendor"
export CARGO_HOME="$scratch/resolve-cache"
export CARGO_TARGET_DIR="$scratch/build-target"
export PATH="$(dirname "$tool"):$PATH"
"$tool" update
"$tool" vendor --locked vendor > .cargo/config.toml
