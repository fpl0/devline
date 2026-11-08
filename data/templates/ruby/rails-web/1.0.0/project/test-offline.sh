#!/bin/sh
set -eu
cd "$(dirname "$0")"
ruby=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-ruby}}
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-rails-test.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
export DEVLINE_GEM_PATH="$scratch/gems" DEVLINE_BUNDLER_HOME="$scratch/bundler"
export RAILS_ENV=test RACK_ENV=test DEVLINE_RAILS_TEST_DIR="$scratch"
unset DATABASE_URL TEST_DATABASE_URL
./bundle-offline.sh install
# Apply the migration from scratch instead of bootstrapping from db/schema.rb.
SCHEMA="$scratch/migration-schema.rb" ./bundle-offline.sh exec "$ruby" bin/rails db:migrate
./bundle-offline.sh exec "$ruby" bin/rails test
