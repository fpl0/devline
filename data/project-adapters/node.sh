#!/bin/sh
# The lock and npm cache stay portable. Node's install/build outputs are local.
set -eu
mode=${1:?prepare, verify, or run}; project=${2:?project}; cache=${3:?managed cache}; tool=${4:?node}; shift 4
. "$(dirname "$0")/common.sh"
cd "$project"
export PATH="$(dirname "$tool"):$PATH"
export npm_config_audit=false npm_config_fund=false npm_config_update_notifier=false CI=true
[ -f package-lock.json ] || { echo 'Commit a package-lock.json first; Devline preserves its exact dependency graph.' >&2; exit 1; }
command -v npm >/dev/null || { echo 'npm must be available alongside the detected Node.js installation.' >&2; exit 1; }
lock_digest=$(/usr/bin/shasum -a 256 package-lock.json | /usr/bin/awk '{print $1}')

use_cache() {
    [ -d "$cache/npm-cache" ] && [ "$(cat "$cache/npm-lock.sha256" 2>/dev/null || :)" = "$lock_digest" ] || {
        echo 'Prepare this package-lock.json before installing its dependencies offline.' >&2
        return 1
    }
    scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-npm.XXXXXX")
    trap 'rm -rf "$scratch"' EXIT
    trap 'exit 129' HUP
    trap 'exit 130' INT
    trap 'exit 143' TERM
    mkdir "$scratch/npm-cache"
    if ! /bin/cp -cR "$cache/npm-cache/." "$scratch/npm-cache/" 2>/dev/null; then
        /bin/cp -R "$cache/npm-cache/." "$scratch/npm-cache/"
    fi
    export npm_config_cache="$scratch/npm-cache" npm_config_offline=true
    if [ -d "$cache/browsers" ]; then export PLAYWRIGHT_BROWSERS_PATH="$cache/browsers"; fi
}

case "$mode" in
  prepare)
    mkdir -p "$cache/npm-cache"
    export npm_config_cache="$cache/npm-cache"
    npm ci --include=dev --no-audit --no-fund
    if [ -f node_modules/playwright/cli.js ]; then
        if { [ -f .devline/BROWSER.json ] && [ -d .devline/packages/browser-archive ] && [ -f .devline/scripts/test-offline.sh ]; } ||
           { [ -f BROWSER.json ] && [ -d browser-archive ] && [ -f test-offline.sh ]; }; then
            # The bundled starter's test restores and checks its pinned browser.
            :
        elif offline_requested; then
            [ -d "$cache/browsers" ] || { echo 'Prepare the pinned browser before adding clients offline.' >&2; exit 1; }
        else
            PLAYWRIGHT_BROWSERS_PATH="$cache/browsers" "$tool" node_modules/playwright/cli.js install
        fi
    fi
    rm -rf "$cache/npm-cache/_logs"
    printf '%s\n' "$lock_digest" > "$cache/npm-lock.sha256"
    ;;
  verify)
    use_cache
    npm ci --offline --include=dev --no-audit --no-fund
    scripts=$("$tool" -e '
        const { scripts } = JSON.parse(require("node:fs").readFileSync("package.json", "utf8"));
        const selected = ["build", "test"].filter(name => scripts?.[name]);
        if (!selected.length) {
            console.error("Add a build or test script so Devline can prove this project works offline.");
            process.exit(1);
        }
        console.log(selected.join(","));
    ')
    case "$scripts" in
      build) npm run build ;;
      test) npm test ;;
      build,test) npm run build; npm test ;;
      *) echo 'Could not determine the Node.js verification scripts.' >&2; exit 1 ;;
    esac
    ;;
  run)
    [ "$#" -gt 0 ] || { echo 'Choose a command, for example devline project run -- npm test.' >&2; exit 1; }
    export npm_config_offline=true
    if [ -d "$cache/npm-cache" ]; then
        use_cache
        if [ "$(cat node_modules/.devline-lock.sha256 2>/dev/null || :)" != "$lock_digest" ]; then
            npm ci --offline --include=dev --no-audit --no-fund >&2
            printf '%s\n' "$lock_digest" > node_modules/.devline-lock.sha256
        fi
        "$@"
    else
        [ -d node_modules ] || { echo 'Prepare the project before restoring its offline Node dependencies.' >&2; exit 1; }
        exec "$@"
    fi
    ;;
  *) echo 'Unknown Node.js adapter action' >&2; exit 2 ;;
esac
