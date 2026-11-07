#!/bin/sh
set -eu
cd "$(dirname "$0")"
if ! /usr/bin/xcrun --no-cache --find xcodebuild >/dev/null 2>&1; then
    printf '%s\n' 'Select Xcode 16 or later to run the bundled Swift Testing suite. Set DEVELOPER_DIR or use xcode-select.' >&2
    exit 1
fi
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-idea-board.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
export DEVLINE_SWIFT_SCRATCH="$scratch/swift"
export DEVLINE_SWIFT_APP_DIR="$scratch/apps"
./scripts/swift-local.sh test --disable-xctest
app=$(./build-app.sh)
/usr/bin/plutil -lint "$app/Contents/Info.plist"
executable=$(/usr/bin/plutil -extract CFBundleExecutable raw "$app/Contents/Info.plist")
test -x "$app/Contents/MacOS/$executable"
/usr/bin/codesign --verify --strict "$app"
printf '%s\n' 'Idea Board tests passed and the local macOS app bundle verified.'
