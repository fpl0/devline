#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$(dirname "$ROOT")"
STAGING_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/devkit-install.XXXXXX")"
trap 'rm -rf "$STAGING_ROOT"' EXIT
ditto -x -k "$OUT/Devline-macos-arm64.zip" "$STAGING_ROOT"
APP="$STAGING_ROOT/Devline.app"
codesign --verify --deep --strict "$APP"
# iCloud File Provider adds Finder metadata to .app directories in Documents,
# which invalidates resource verification. Keep the executable bundle local.
INSTALLED_APP="$HOME/Applications/Devline.app"
LEGACY_APP="$HOME/Applications/DevKit.app"
if [ ! -e "$INSTALLED_APP" ] && [ ! -L "$INSTALLED_APP" ] && [ -d "$LEGACY_APP" ] && [ ! -L "$LEGACY_APP" ]; then
    LEGACY_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$LEGACY_APP/Contents/Info.plist" 2>/dev/null || true)
    if [ "$LEGACY_ID" = 'io.devkit.desktop' ]; then
        mv "$LEGACY_APP" "$INSTALLED_APP"
    fi
fi
if [ -e "$INSTALLED_APP" ] || [ -L "$INSTALLED_APP" ]; then
    [ ! -L "$INSTALLED_APP" ] || { echo 'Refusing to replace an Applications symlink.' >&2; exit 1; }
    EXISTING_ID=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$INSTALLED_APP/Contents/Info.plist" 2>/dev/null || true)
    [ "$EXISTING_ID" = 'io.devkit.desktop' ] || { echo 'A different app already occupies the install location.' >&2; exit 1; }
fi
# Replacing a mapped executable in place can terminate the running app or CLI.
# Keep the current installation intact until its processes have exited.
python3 - "$INSTALLED_APP" <<'PY'
import os
import subprocess
import sys

executables = os.path.realpath(sys.argv[1]) + "/Contents/MacOS/"
processes = subprocess.check_output(["/bin/ps", "-axo", "pid=,comm="], text=True)
for line in processes.splitlines():
    fields = line.strip().split(None, 1)
    if len(fields) == 2 and os.path.realpath(fields[1]).startswith(executables):
        raise SystemExit("Quit Devline and let its active CLI jobs finish, then run the installer again. Your installed app has not been changed.")
PY
mkdir -p "$INSTALLED_APP"
rsync -a --delete "$APP/" "$INSTALLED_APP/"
codesign --verify --deep --strict "$INSTALLED_APP"
# Retarget only a CLI link created by the previous app, preserving unrelated commands.
LEGACY_CLI="$HOME/.local/bin/devkit"
if [ -L "$LEGACY_CLI" ] && [ "$(readlink "$LEGACY_CLI")" = "$LEGACY_APP/Contents/MacOS/devkit" ]; then
    ln -sfn "$INSTALLED_APP/Contents/MacOS/devkit" "$LEGACY_CLI"
fi
touch "$INSTALLED_APP"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$INSTALLED_APP"
printf 'Installed %s\n' "$INSTALLED_APP"
