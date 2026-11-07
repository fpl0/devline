#!/bin/sh
# Devline runs this recipe in a staging copy, then verifies it without egress.
set -eu
project=${1:?project folder}
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-tauri-refresh.XXXXXXXX")
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' INT
trap 'exit 143' HUP TERM

tool=${DEVKIT_TOOLCHAIN:-cargo}
cd "$project"
mv .cargo/config.toml "$scratch/previous-config"
mv vendor "$scratch/previous-vendor"
export CARGO_HOME="$scratch/resolve-cache"
export PATH="$(dirname "$tool"):$PATH"
"$tool" update
"$tool" vendor --locked vendor > .cargo/config.toml
