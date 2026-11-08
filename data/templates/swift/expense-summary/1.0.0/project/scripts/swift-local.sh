#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
command=${1:?Choose a SwiftPM command}
shift
scratch=${DEVLINE_SWIFT_SCRATCH:-"$PWD/.build/devline-swift"}
swift=${DEVKIT_TOOLCHAIN:-$(/usr/bin/xcrun --no-cache --find swift 2>/dev/null || true)}
if [ -z "$swift" ] || ! sdk=$(/usr/bin/xcrun --no-cache --sdk macosx --show-sdk-path 2>/dev/null) || [ ! -d "$sdk" ]; then
    printf '%s\n' 'Select a local Xcode installation with Swift 6 and the macOS SDK.' >&2
    exit 1
fi
export SDKROOT="$sdk"
export CLANG_MODULE_CACHE_PATH="$scratch/clang-cache"
export SWIFTPM_MODULECACHE_OVERRIDE="$scratch/manifest-modules"
mkdir -p "$scratch"
# Devline supplies the outer sandbox during verification. This package uses only local targets.
exec "$swift" "$command" --disable-sandbox --disable-automatic-resolution \
    --disable-dependency-cache --manifest-cache local --build-system native \
    --scratch-path "$scratch/build" --cache-path "$scratch/cache" \
    --config-path "$scratch/config" --security-path "$scratch/security" "$@"
