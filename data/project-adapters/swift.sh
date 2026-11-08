#!/bin/sh
set -eu
action=${1:?prepare, verify, or run}
project=${2:?project directory}
managed=${3:?managed dependency directory}/swift
tool=${4:?Swift executable}
shift 4
. "$(dirname "$0")/common.sh"
cd "$project"
[ -f Package.swift ] || { echo 'Choose the Swift package directory containing Package.swift.' >&2; exit 1; }
if ! sdk=$(/usr/bin/xcrun --no-cache --sdk macosx --show-sdk-path 2>/dev/null) || [ ! -d "$sdk" ]; then
    echo 'Select an installed macOS SDK with xcode-select before preparing this project.' >&2
    exit 1
fi
export PATH="$(dirname "$tool"):$PATH" SDKROOT="$sdk"
export_starter_toolchain "$tool"
case "$action" in
    prepare) scratch=${TMPDIR:?}/swift-prepare; cache="$managed/cache" ;;
    verify) scratch=${TMPDIR:?}/swift-verify; cache="$scratch/cache" ;;
    run) scratch="$project/.build/devline-adapter"; cache="$scratch/cache" ;;
    *) echo 'Expected prepare, verify, or run.' >&2; exit 2 ;;
esac
mkdir -p "$scratch" "$cache"
if [ "$action" != prepare ] && [ -d "$managed/cache/repositories" ]; then
    # Clone only source repositories. Builds and manifests belong to this invocation.
    mkdir -p "$cache/repositories"
    /bin/cp -cR "$managed/cache/repositories/." "$cache/repositories/"
fi
export CLANG_MODULE_CACHE_PATH="$scratch/clang-cache" SWIFTPM_MODULECACHE_OVERRIDE="$scratch/manifest-modules"
swift_local() {
    verb=$1; shift
    "$tool" "$verb" --disable-sandbox --manifest-cache local --build-system native \
        --scratch-path "$scratch/build" --cache-path "$cache" \
        --config-path "$scratch/config" --security-path "$scratch/security" "$@"
}
package=$(swift_local package dump-package)
if [ "$action" = prepare ]; then
    previous="$scratch/previous-resolved.json"
    if [ -f Package.resolved ]; then cp Package.resolved "$previous"; fi
    if offline_requested; then
        swift_local package --skip-update resolve
    else
        swift_local package resolve
    fi
    if [ -f "$previous" ] && [ -f Package.resolved ]; then
        python3 - "$previous" Package.resolved <<'PYTHON'
import json,sys
old,new=[{p['identity']:p['state'] for p in json.load(open(path)).get('pins',[])} for path in sys.argv[1:]]
changed=[name for name,state in old.items() if name in new and new[name]!=state]
if changed: raise SystemExit('Adding or preparing clients would change existing Swift pins: '+', '.join(changed))
PYTHON
    fi
    exit
fi
# Resolve only from the lock and the prepared source cache. The caller also denies
# outbound networking, so an incomplete archive becomes an actionable failure.
swift_local package --skip-update --force-resolved-versions resolve
if [ "$action" = run ]; then
    [ "$#" -gt 0 ] || { echo 'Specify a command to run in the Swift project.' >&2; exit 2; }
    case "${1##*/}:${2:-}" in
        swift:build|swift:test|swift:run|swift:package)
            shift
            verb=$1; shift
            swift_local "$verb" --skip-update --disable-automatic-resolution "$@"
            ;;
        *) exec "$@" ;;
    esac
else
    has_tests=$(printf '%s' "$package" | python3 -c 'import json,sys;print(any(t["type"]=="test" for t in json.load(sys.stdin)["targets"]))')
    if [ "$has_tests" = True ]; then
        swift_local test --skip-update --disable-automatic-resolution -j 4
    else
        swift_local build --skip-update --disable-automatic-resolution -j 4
    fi
fi
