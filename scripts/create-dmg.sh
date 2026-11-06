#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
outputs="$(dirname "$root")"
zip="${1:-$outputs/Devline-macos-arm64.zip}"
output="${2:-$outputs/Devline-macos-arm64.dmg}"

if [[ $# -gt 2 || ${1:-} == --help ]]; then
    printf 'Usage: %s [Devline-macos-arm64.zip [Devline-macos-arm64.dmg]]\n' "$0"
    exit "$([[ $# -gt 2 ]] && printf 2 || printf 0)"
fi

[[ -f "$zip" ]] || { printf 'Devline ZIP not found at %s\n' "$zip" >&2; exit 1; }
case "$output" in
    *.dmg) ;;
    *) printf 'The output filename must end in .dmg\n' >&2; exit 1 ;;
esac
if [[ -L "$output" || ( -e "$output" && ! -f "$output" ) ]]; then
    printf 'Choose a regular output file at %s\n' "$output" >&2
    exit 1
fi

zip="$(cd "$(dirname "$zip")" && pwd)/$(basename "$zip")"
output_dir="$(cd "$(dirname "$output")" && pwd)"
output="$output_dir/$(basename "$output")"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/devline-dmg.XXXXXX")"
publish_dir=""
mount_point="$scratch/mounted"
mounted=false

cleanup() {
    if "$mounted"; then
        if ! /usr/bin/hdiutil detach -quiet -force "$mount_point"; then
            printf 'Eject the image before removing %s\n' "$scratch" >&2
            [[ -z "$publish_dir" ]] || rm -rf "$publish_dir"
            return
        fi
    fi
    rm -rf "$scratch"
    [[ -z "$publish_dir" ]] || rm -rf "$publish_dir"
}
trap cleanup EXIT

publish_dir="$(mktemp -d "$output_dir/.devline-dmg.XXXXXX")"
contents="$scratch/contents"
extracted="$scratch/extracted"
mkdir "$contents" "$extracted" "$mount_point"
/usr/bin/ditto -x -k "$zip" "$extracted"

validate_bundle() {
    local bundle="$1" info="$1/Contents/Info.plist" name id executable
    [[ -d "$bundle" && ! -L "$bundle" && -f "$info" ]] || {
        printf 'Expected a real Devline.app bundle at %s\n' "$bundle" >&2; return 1;
    }
    name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleDisplayName' "$info")"
    id="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$info")"
    executable="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$info")"
    [[ "$name" == Devline && "$id" == io.devkit.desktop && "$executable" != */* ]] || {
        printf 'The ZIP does not contain the current Devline app.\n' >&2; return 1;
    }
    [[ -x "$bundle/Contents/MacOS/devline" ]] || {
        printf 'The bundled devline command is missing.\n' >&2; return 1;
    }
    [[ "$(/usr/bin/lipo -archs "$bundle/Contents/MacOS/$executable")" == arm64 ]] || {
        printf 'Expected an Apple Silicon executable.\n' >&2; return 1;
    }
    /usr/bin/codesign --verify --deep --strict "$bundle"
}

validate_bundle "$extracted/Devline.app"
app="$contents/Devline.app"
mv "$extracted/Devline.app" "$app"
version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app/Contents/Info.plist")"
ln -s /Applications "$contents/Applications"
cat > "$contents/Read Me.txt" <<README
Devline $version
Apple Silicon Mac with macOS 14 or later

1. Drag Devline into Applications.
2. Open Devline from Applications.
3. Eject the Devline disk image when the copy completes.

Terminal setup

In Devline, open Settings → Command Line and select Install CLI.
If your shell cannot find devline, add the CLI folder to its PATH.

    export PATH="\$HOME/.local/bin:\$PATH"
    devline --version
README

signature="$(/usr/bin/codesign -d --verbose=2 "$app" 2>&1)"
if [[ "$signature" == *Signature=adhoc* ]]; then
    cat >> "$contents/Read Me.txt" <<'README'

First launch

This build is ad hoc signed. If Gatekeeper blocks the first launch, follow
Apple’s guidance for a download you trust.

https://support.apple.com/en-us/102445
README
fi

image="$publish_dir/Devline.dmg"
/usr/bin/hdiutil create -volname Devline -srcfolder "$contents" \
    -fs HFS+ -nospotlight -format UDZO -imagekey zlib-level=9 "$image"
/usr/bin/hdiutil verify -quiet "$image"
/usr/bin/hdiutil attach -quiet -readonly -mount required -noautoopen -mountpoint "$mount_point" "$image"
mounted=true
validate_bundle "$mount_point/Devline.app"
[[ "$(readlink "$mount_point/Applications")" == /Applications ]]
[[ -f "$mount_point/Read Me.txt" ]]
printf 'Verified the mounted app signature and installer contents\n'
/usr/bin/hdiutil detach -quiet "$mount_point"
mounted=false

mv -f "$image" "$output"
printf 'Verified installer %s\n' "$output"
/usr/bin/shasum -a 256 "$output"
