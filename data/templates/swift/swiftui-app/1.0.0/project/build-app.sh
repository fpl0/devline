#!/bin/sh
set -eu
cd "$(dirname "$0")"
name='__PROJECT_NAME__'
case "$name" in
    ''|*[!a-z0-9-]*) name=fieldnotes ;;
esac
app_dir=${DEVLINE_SWIFT_APP_DIR:-"$PWD/.build/apps"}
mkdir -p "$app_dir"
app_dir=$(cd "$app_dir" && pwd)
app="$app_dir/$name.app"
stage=$(mktemp -d "$app_dir/.idea-board.XXXXXX")
trap 'rm -rf "$stage"' EXIT
trap 'exit 130' HUP INT TERM

./scripts/swift-local.sh build --configuration release --product IdeaBoard >&2
bin_path=$(./scripts/swift-local.sh build --configuration release --show-bin-path)
bundle="$stage/$name.app"
mkdir -p "$bundle/Contents/MacOS"
/usr/bin/install -m 755 "$bin_path/IdeaBoard" "$bundle/Contents/MacOS/IdeaBoard"
plist="$bundle/Contents/Info.plist"
/usr/bin/plutil -create xml1 "$plist"
/usr/bin/plutil -insert CFBundleIdentifier -string "local.devline.$name" "$plist"
/usr/bin/plutil -insert CFBundleName -string "$name" "$plist"
/usr/bin/plutil -insert CFBundleDisplayName -string "Fieldnotes" "$plist"
/usr/bin/plutil -insert CFBundleExecutable -string "IdeaBoard" "$plist"
/usr/bin/plutil -insert CFBundlePackageType -string APPL "$plist"
/usr/bin/plutil -insert CFBundleShortVersionString -string "1.0.0" "$plist"
/usr/bin/plutil -insert CFBundleVersion -string "1" "$plist"
/usr/bin/plutil -insert LSMinimumSystemVersion -string "14.0" "$plist"
/usr/bin/plutil -insert NSHighResolutionCapable -bool YES "$plist"
# Files in synced folders can carry metadata that macOS excludes from signed bundles.
/usr/bin/xattr -dr com.apple.FinderInfo "$bundle" 2>/dev/null || true
/usr/bin/xattr -dr com.apple.ResourceFork "$bundle" 2>/dev/null || true
/usr/bin/codesign --force --sign - --timestamp=none "$bundle" >&2
/usr/bin/codesign --verify --strict "$bundle"

rm -rf "$app"
mv "$bundle" "$app"
printf '%s\n' "$app"
