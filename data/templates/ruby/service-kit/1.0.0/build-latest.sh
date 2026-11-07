#!/bin/sh
# Online refresh. Devline tests the result offline before installing a new release.
set -eu
project=${1:?project folder}
ruby=${DEVKIT_TOOLCHAIN:-ruby}
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
cd "$project"
export BUNDLE_IGNORE_CONFIG=1 BUNDLE_GEMFILE="$project/Gemfile"
export BUNDLE_PATH="$scratch/gems" BUNDLE_USER_HOME="$scratch/bundler" BUNDLE_APP_CONFIG="$scratch/config"
"$ruby" -S bundle lock --update --add-checksums
"$ruby" -S bundle cache --no-install
