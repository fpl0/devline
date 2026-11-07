#!/bin/sh
set -eu
. "$(dirname "$0")/common.sh"
cd "$project"
cache="$project/.devline-offline/npm-cache"
merge "$artifacts/npm-cache" "$cache"
export npm_config_cache="$cache" npm_config_audit=false npm_config_fund=false npm_config_update_notifier=false CI=true
if [ "$offline" = true ]; then export npm_config_offline=true; else export npm_config_prefer_offline=true; fi
add_package() { npm install --save-exact --include=dev --no-audit --no-fund "$package_name@$package_version"; }
packages
exec /bin/sh "$catalog/project-adapters/node.sh" prepare "$project" "$project/.devline-offline" "$tool"
