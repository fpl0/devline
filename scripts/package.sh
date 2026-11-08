#!/bin/bash
set -euo pipefail
export MACOSX_DEPLOYMENT_TARGET=14.0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$(dirname "$ROOT")"
SWIFT_BUILD_PATH="${DEVLINE_SWIFT_BUILD_PATH:-$ROOT/macos/.build}"
python3 "$ROOT/scripts/sync-catalog.py"
python3 "$ROOT/scripts/seal-service-client-packs.py" --check
cargo build --release --locked --manifest-path "$ROOT/core/Cargo.toml"
APP_VERSION="$(cargo metadata --offline --no-deps --format-version 1 --manifest-path "$ROOT/core/Cargo.toml" | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "devline"))')"
# SwiftPM does not track this externally built static library as a source dependency.
# Invalidate its build when the Rust archive changes so the app cannot ship a stale core.
CORE_DIGEST="$(shasum -a 256 "$ROOT/core/target/release/libdevline_core.a" | cut -d ' ' -f 1)"
PREVIOUS_DIGEST="$(cat "$SWIFT_BUILD_PATH/devline-core.sha256" 2>/dev/null || true)"
if [ "$CORE_DIGEST" != "$PREVIOUS_DIGEST" ]; then
    swift package --package-path "$ROOT/macos" --scratch-path "$SWIFT_BUILD_PATH" clean
fi
swift build --package-path "$ROOT/macos" --scratch-path "$SWIFT_BUILD_PATH" --build-system native -c release
printf '%s\n' "$CORE_DIGEST" > "$SWIFT_BUILD_PATH/devline-core.sha256"
STAGING_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/devline-build.XXXXXX")"
ARCHIVE_STAGE=""
cleanup() {
    rm -rf "$STAGING_ROOT"
    [[ -z "$ARCHIVE_STAGE" ]] || rm -rf "$ARCHIVE_STAGE"
}
trap cleanup EXIT
APP="$STAGING_ROOT/Devline.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$APP/Contents/Frameworks"
cp "$SWIFT_BUILD_PATH/release/DevlineApp" "$APP/Contents/MacOS/DevlineApp"
ditto "$SWIFT_BUILD_PATH/release/DevlineMac_DevlineApp.bundle" "$APP/Contents/Resources/DevlineMac_DevlineApp.bundle"
ditto "$SWIFT_BUILD_PATH/release/Sparkle.framework" "$APP/Contents/Frameworks/Sparkle.framework"
cp "$ROOT/core/target/release/devline" "$APP/Contents/MacOS/devline"
ln -s devline "$APP/Contents/MacOS/devkit"
xcrun actool "$ROOT/macos/Assets/Devline.icon" \
    --compile "$APP/Contents/Resources" --platform macosx \
    --minimum-deployment-target 14.0 --app-icon Devline \
    --output-partial-info-plist "$STAGING_ROOT/icon-info.plist"
rsync -a --delete --exclude='.DS_Store' --exclude='__pycache__/' --exclude='*.pyc' --exclude='*.pyo' \
    --exclude='/.catalog-*/' --exclude='/.client-seal-*/' --exclude='/docs/' --exclude='/templates/**/.gradle/' \
    "$ROOT/data/" "$APP/Contents/Resources/catalog/"
python3 "$ROOT/scripts/seal-service-client-packs.py" --check --data "$APP/Contents/Resources/catalog"
python3 - "$ROOT/data" "$APP/Contents/Resources/catalog" <<'PYTHON'
import json
from pathlib import Path
import shutil
import sys

source, destination = map(Path, sys.argv[1:])
for pack in json.loads((source / 'catalog.json').read_text())['docs']:
    if pack['id'] == 'swiftui' or pack.get('local_only', False) is not False:
        raise SystemExit('Local SDK documentation cannot be bundled in the application')
    relative = Path(pack['path'])
    original = source / relative
    path = original.resolve()
    expected = Path('docs') / pack['id'] / pack['version']
    if relative != expected or relative.is_absolute() or not path.is_relative_to((source / 'docs').resolve()):
        raise SystemExit('Documentation path escapes the bundled reference catalog')
    if any((source / Path(*relative.parts[:length])).is_symlink() for length in range(1, len(relative.parts) + 1)):
        raise SystemExit('Documentation pack paths must not contain symbolic links')
    for entry in path.rglob('*'):
        if entry.is_symlink() or not (entry.is_dir() or entry.is_file()):
            raise SystemExit('Documentation packs must contain only regular files and directories')
    shutil.copytree(path, destination / relative, ignore=shutil.ignore_patterns('.DS_Store'))
PYTHON
cp "$ROOT/THIRD_PARTY_NOTICES.md" "$APP/Contents/Resources/THIRD_PARTY_NOTICES.md"
rsync -a --exclude='.DS_Store' "$ROOT/third-party/" "$APP/Contents/Resources/third-party/"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>DevlineApp</string>
<key>CFBundleIdentifier</key><string>io.devkit.desktop</string>
<key>CFBundleName</key><string>Devline</string>
<key>CFBundleDisplayName</key><string>Devline</string>
<key>CFBundleIconFile</key><string>Devline</string>
<key>CFBundleIconName</key><string>Devline</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.2.0</string>
<key>CFBundleVersion</key><string>3</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
<key>CFBundleURLTypes</key><array><dict><key>CFBundleURLName</key><string>Devline reference</string><key>CFBundleURLSchemes</key><array><string>devline</string><string>devkit</string></array></dict></array>
<key>NSApplicationServicesProvider</key><true/>
<key>NSServices</key><array><dict>
<key>NSMessage</key><string>lookUpInDevline</string>
<key>NSMenuItem</key><dict><key>default</key><string>Look Up in Devline</string></dict>
<key>NSSendTypes</key><array><string>NSStringPboardType</string><string>public.utf8-plain-text</string></array>
<key>NSPortName</key><string>Devline</string>
</dict></array>
<key>SUPublicEDKey</key><string>VMLqt73XeFU7d/CqPBorEl6j6zDLn+d66lPb2MsLiro=</string>
<key>SUFeedURL</key><string>https://github.com/fpl0/devline/releases/latest/download/appcast.xml</string>
<key>SURequireSignedFeed</key><true/>
<key>SUVerifyUpdateBeforeExtraction</key><true/>
<key>SUEnableAutomaticChecks</key><false/>
<key>DevlineSignedUpdatesEnabled</key><false/>
<key>SUAutomaticallyUpdate</key><false/>
<key>SUSendProfileInfo</key><false/>
<key>SUScheduledCheckInterval</key><integer>86400</integer>
</dict></plist>
PLIST
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $APP_VERSION" "$APP/Contents/Info.plist"
APP_BUILD="$(python3 -c 'import sys; a,b,c=map(int,sys.argv[1].split(".")); print(a*1000000+b*1000+c)' "$APP_VERSION")"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $APP_BUILD" "$APP/Contents/Info.plist"
# Finder may attach presentation metadata when a development bundle is opened.
# Remove only the metadata disallowed by codesign; preserve security attributes.
xattr -dr com.apple.FinderInfo "$APP" 2>/dev/null || true
xattr -dr com.apple.ResourceFork "$APP" 2>/dev/null || true
# Sign nested executables from the inside out; preserve Sparkle's framework links.
SIGN_IDENTITY="${DEVLINE_SIGN_IDENTITY:--}"
for nested in \
    "$APP/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Installer.xpc" \
    "$APP/Contents/Frameworks/Sparkle.framework/Versions/B/XPCServices/Downloader.xpc" \
    "$APP/Contents/Frameworks/Sparkle.framework/Versions/B/Autoupdate" \
    "$APP/Contents/Frameworks/Sparkle.framework/Versions/B/Updater.app" \
    "$APP/Contents/Frameworks/Sparkle.framework" \
    "$APP/Contents/MacOS/devline" \
    "$APP"; do
    codesign --force --sign "$SIGN_IDENTITY" "$nested"
done
codesign --verify --deep --strict "$APP"
ARCHIVE_STAGE="$(mktemp -d "$OUT/.devline-package.XXXXXX")"
ditto --norsrc -c -k --keepParent "$APP" "$ARCHIVE_STAGE/Devline-macos-arm64.zip"
python3 - "$ARCHIVE_STAGE/Devline-macos-arm64.zip" "$OUT/Devline-macos-arm64.zip" <<'PYTHON'
import os
from pathlib import Path
import sys

source, destination = map(Path, sys.argv[1:])
if destination.is_symlink() or (destination.exists() and not destination.is_file()):
    raise SystemExit("Refusing to replace a non-regular ZIP output.")
os.replace(source, destination)
PYTHON
if [ "${1:-}" != "--no-install" ]; then
    bash "$ROOT/scripts/install.sh"
fi
INSTALLED_APP="$HOME/Applications/Devline.app"
if [ -d "$OUT/Devline.app" ] && [ ! -L "$OUT/Devline.app" ]; then
    EXISTING_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$OUT/Devline.app/Contents/Info.plist" 2>/dev/null || true)
    [ "$EXISTING_ID" = 'io.devkit.desktop' ] || { echo 'Refusing to replace an unrelated output.' >&2; exit 1; }
    rm -rf "$OUT/Devline.app"
fi
ln -sfn "$INSTALLED_APP" "$OUT/Devline.app"
mkdir -p "$OUT/bin"
cat > "$OUT/bin/devline" <<'SH'
#!/bin/sh
exec "$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)/../Devline.app/Contents/MacOS/devline" "$@"
SH
chmod +x "$OUT/bin/devline"
ln -sfn devline "$OUT/bin/devkit"
printf 'Built %s\n' "$OUT/Devline.app"
