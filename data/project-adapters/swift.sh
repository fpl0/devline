#!/bin/sh
set -eu
action=${1:?prepare, verify, or run}
project=${2:?project directory}
tool=${4:?Swift executable}
shift 4
cd "$project"
if [ ! -f Package.swift ]; then
    printf '%s\n' 'Choose the Swift package directory containing Package.swift.' >&2
    exit 1
fi
if ! sdk=$(/usr/bin/xcrun --no-cache --sdk macosx --show-sdk-path 2>/dev/null) || [ ! -d "$sdk" ]; then
    printf '%s\n' 'Select an installed macOS SDK with xcode-select before preparing this project.' >&2
    exit 1
fi
export PATH="$(dirname "$tool"):$PATH"
export DEVKIT_TOOLCHAIN="$tool" SDKROOT="$sdk"

swift_local() {
    verb=$1
    shift
    "$tool" "$verb" --disable-sandbox --disable-automatic-resolution \
        --disable-dependency-cache --manifest-cache local --build-system native \
        --scratch-path "$scratch/build" --cache-path "$scratch/cache" \
        --config-path "$scratch/config" --security-path "$scratch/security" "$@"
}

manifest() {
    package=$(swift_local package dump-package)
    dependencies=$(printf '%s' "$package" | /usr/bin/plutil -extract dependencies raw -expect array -o - -)
    if [ "$dependencies" -ne 0 ]; then
        printf '%s\n' 'Offline preparation currently supports Swift packages without package dependencies. Use a dependency-free package such as the SwiftUI starter.' >&2
        exit 1
    fi
}

case "$action" in
    prepare)
        # The verification phase evaluates the manifest and builds with the network denied.
        exit 0
        ;;
    verify)
        scratch=${TMPDIR:?}/swiftpm
        ;;
    run)
        [ "$#" -gt 0 ] || { printf '%s\n' 'Specify a command to run in the Swift project.' >&2; exit 2; }
        scratch="$project/.build/devline-adapter"
        ;;
    *)
        printf '%s\n' 'Expected prepare, verify, or run.' >&2
        exit 2
        ;;
esac
mkdir -p "$scratch"
export CLANG_MODULE_CACHE_PATH="$scratch/clang-cache"
export SWIFTPM_MODULECACHE_OVERRIDE="$scratch/manifest-modules"

if [ "$action" = run ]; then
    case "${1##*/}:${2:-}" in
        swift:build|swift:test|swift:run|swift:package)
            shift
            manifest
            swift_local "$@"
            ;;
        *) exec "$@" ;;
    esac
else
    manifest
    count=$(printf '%s' "$package" | /usr/bin/plutil -extract targets raw -expect array -o - -)
    index=0
    while [ "$index" -lt "$count" ]; do
        type=$(printf '%s' "$package" | /usr/bin/plutil -extract "targets.$index.type" raw -expect string -o - -)
        if [ "$type" = test ]; then
            swift_local test
            exit
        fi
        index=$((index + 1))
    done
    swift_local build
fi
