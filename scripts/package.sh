#!/bin/bash
set -euo pipefail
export MACOSX_DEPLOYMENT_TARGET=14.0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$(dirname "$ROOT")"
python3 - "$ROOT" <<'PY'
import json,sys
from pathlib import Path
root=Path(sys.argv[1])
data={"schema_version":1,"docs":json.loads((root/'data/docs-catalog.json').read_text()),"templates":json.loads((root/'data/templates-catalog.json').read_text())}
(root/'data/catalog.json').write_text(json.dumps(data,indent=2)+'\n')
PY
cargo build --release --locked --manifest-path "$ROOT/core/Cargo.toml"
APP_VERSION="$(cargo metadata --offline --no-deps --format-version 1 --manifest-path "$ROOT/core/Cargo.toml" | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "devkit"))')"
# SwiftPM does not track this externally built static library as a source dependency.
# Invalidate its build when the Rust archive changes so the app cannot ship a stale core.
CORE_DIGEST="$(shasum -a 256 "$ROOT/core/target/release/libdevkit_core.a" | cut -d ' ' -f 1)"
PREVIOUS_DIGEST="$(cat "$ROOT/macos/.build/devline-core.sha256" 2>/dev/null || true)"
if [ "$CORE_DIGEST" != "$PREVIOUS_DIGEST" ]; then
    swift package --package-path "$ROOT/macos" clean
fi
swift build --package-path "$ROOT/macos" --build-system native -c release
printf '%s\n' "$CORE_DIGEST" > "$ROOT/macos/.build/devline-core.sha256"
STAGING_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/devkit-build.XXXXXX")"
trap 'rm -rf "$STAGING_ROOT"' EXIT
APP="$STAGING_ROOT/Devline.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$ROOT/macos/.build/release/DevKitApp" "$APP/Contents/MacOS/DevlineApp"
cp "$ROOT/core/target/release/devline" "$APP/Contents/MacOS/devline"
ln -s devline "$APP/Contents/MacOS/devkit"
xcrun actool "$ROOT/macos/Assets/DevKit.icon" \
    --compile "$APP/Contents/Resources" --platform macosx \
    --minimum-deployment-target 14.0 --app-icon DevKit \
    --output-partial-info-plist "$STAGING_ROOT/icon-info.plist"
rsync -a --delete --exclude='.DS_Store' "$ROOT/data/" "$APP/Contents/Resources/catalog/"
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
<key>CFBundleIconFile</key><string>DevKit</string>
<key>CFBundleIconName</key><string>DevKit</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.2.0</string>
<key>CFBundleVersion</key><string>2</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
<key>CFBundleURLTypes</key><array><dict><key>CFBundleURLName</key><string>Devline reference</string><key>CFBundleURLSchemes</key><array><string>devline</string><string>devkit</string></array></dict></array>
</dict></plist>
PLIST
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $APP_VERSION" "$APP/Contents/Info.plist"
# Finder may attach presentation metadata when a development bundle is opened.
# Remove only the metadata disallowed by codesign; preserve security attributes.
xattr -dr com.apple.FinderInfo "$APP" 2>/dev/null || true
xattr -dr com.apple.ResourceFork "$APP" 2>/dev/null || true
codesign --force --deep --sign - "$APP"
codesign --verify --deep --strict "$APP"
ditto --norsrc -c -k --keepParent "$APP" "$OUT/Devline-macos-arm64.zip"
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
