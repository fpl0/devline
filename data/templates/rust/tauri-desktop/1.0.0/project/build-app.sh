#!/bin/sh
set -eu
cd "$(dirname "$0")"
export MACOSX_DEPLOYMENT_TARGET=14.0
cargo_target=${CARGO_TARGET_DIR:-target}
output="$cargo_target/bundle/macos"
info=packaging/Info.plist
bundle_id=$(/usr/bin/plutil -extract CFBundleIdentifier raw "$info")
app_title=$(/usr/bin/plutil -extract CFBundleName raw "$info")
case "$bundle_id" in
    '' | *[!A-Za-z0-9.-]*) printf 'Invalid CFBundleIdentifier\n' >&2; exit 1 ;;
esac
case "$app_title" in
    '' | . | .. | */* | *:*) printf 'Invalid CFBundleName\n' >&2; exit 1 ;;
esac
app_name="$app_title.app"
destination="$output/$app_name"
lock="$output/.publish.lock"
mkdir -p "$output"
work=$(mktemp -d "$output/.tauri-app.XXXXXXXX")
locked=false
cleanup() {
    if [ "$locked" = true ]; then
        if [ -d "$work/previous.app" ] && [ ! -e "$destination" ] && [ ! -L "$destination" ] &&
            ! mv "$work/previous.app" "$destination"; then
            printf 'Previous bundle preserved at %s\n' "$work/previous.app" >&2
            rmdir "$lock"
            return
        fi
        rmdir "$lock"
    fi
    rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' HUP TERM

"${DEVKIT_TOOLCHAIN:-cargo}" build --release --frozen --jobs "${CARGO_BUILD_JOBS:-2}" \
    --target-dir "$cargo_target" --target aarch64-apple-darwin --bin tauri-desktop
app="$work/$app_name"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
install -m 755 "$cargo_target/aarch64-apple-darwin/release/tauri-desktop" "$app/Contents/MacOS/tauri-desktop"
install -m 644 "$info" "$app/Contents/Info.plist"
install -m 644 icons/Inspector.icns "$app/Contents/Resources/Inspector.icns"
install -m 644 THIRD_PARTY_NOTICES.md MPL-2.0.txt "$app/Contents/Resources/"
/usr/bin/codesign --force --sign - "$app"
/usr/bin/codesign --verify --strict "$app"
if ! mkdir "$lock" 2>/dev/null; then
    printf 'Another build is publishing, or an interrupted build left %s\n' "$lock" >&2
    exit 1
fi
locked=true
if [ -e "$destination" ] || [ -L "$destination" ]; then
    if [ -L "$destination" ] || [ ! -d "$destination" ] ||
        [ "$(/usr/bin/plutil -extract CFBundleIdentifier raw "$destination/Contents/Info.plist" 2>/dev/null)" != "$bundle_id" ]; then
        printf 'Refusing to replace %s\n' "$destination" >&2
        exit 1
    fi
    mv "$destination" "$work/previous.app"
fi
mv "$app" "$destination"
printf '%s\n' "$destination"
