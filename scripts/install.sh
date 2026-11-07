#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$(dirname "$ROOT")"
APPLICATIONS="$HOME/Applications"
mkdir -p "$APPLICATIONS"
[[ -d "$APPLICATIONS" && ! -L "$APPLICATIONS" ]] || {
    echo 'Choose a real Applications directory before installing Devline.' >&2
    exit 1
}
STAGING_ROOT="$(mktemp -d "$APPLICATIONS/.devline-install.XXXXXX")"
cleanup() {
    if [[ -f "$STAGING_ROOT/.preserve" ]]; then
        printf 'Installer recovery files remain at %s\n' "$STAGING_ROOT" >&2
    else
        rm -rf "$STAGING_ROOT"
    fi
}
trap cleanup EXIT
/usr/bin/ditto -x -k "$OUT/Devline-macos-arm64.zip" "$STAGING_ROOT"
APP="$STAGING_ROOT/Devline.app"
INFO="$APP/Contents/Info.plist"
[[ -d "$APP" && ! -L "$APP" && -f "$INFO" ]] || {
    echo 'The ZIP must contain a real Devline.app bundle.' >&2
    exit 1
}
id=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$INFO")
name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleDisplayName' "$INFO")
executable=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$INFO")
[[ "$id" == io.devkit.desktop && "$name" == Devline && -n "$executable" && "$executable" != */* ]] || {
    echo 'The ZIP does not contain the current Devline app.' >&2
    exit 1
}
[[ -x "$APP/Contents/MacOS/$executable" && -x "$APP/Contents/MacOS/devline" &&
   "$(/usr/bin/lipo -archs "$APP/Contents/MacOS/$executable")" == arm64 &&
   "$(/usr/bin/lipo -archs "$APP/Contents/MacOS/devline")" == arm64 ]] || {
    echo 'Expected Apple Silicon app and CLI executables.' >&2
    exit 1
}
# Clean presentation metadata only from the disposable extracted copy.
/usr/bin/xattr -dr com.apple.FinderInfo "$APP" 2>/dev/null || true
/usr/bin/xattr -dr com.apple.ResourceFork "$APP" 2>/dev/null || true
/usr/bin/codesign --verify --deep --strict "$APP"

# Work on the same volume so a verified bundle can replace the old one in one rename.
python3 - "$APPLICATIONS" "$APP" <<'PY'
import ctypes
import fcntl
import os
from pathlib import Path
import plistlib
import subprocess
import sys

applications, candidate = map(Path, sys.argv[1:])
installed = applications / "Devline.app"
legacy = applications / "DevKit.app"
retired = candidate.parent / "previous-legacy.app"
registration = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
rename_at = ctypes.CDLL(None, use_errno=True).renameatx_np
rename_at.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
AT_FDCWD = -2
RENAME_SWAP = 0x00000002
RENAME_EXCL = 0x00000004


def rename(source, destination, flags):
    if rename_at(AT_FDCWD, os.fsencode(source), AT_FDCWD, os.fsencode(destination), flags) != 0:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), str(destination))


def bundle_id(path):
    if path.is_symlink() or not path.is_dir():
        return None
    try:
        with (path / "Contents/Info.plist").open("rb") as info:
            return plistlib.load(info).get("CFBundleIdentifier")
    except (OSError, ValueError, plistlib.InvalidFileException):
        return None


def require_stopped(*bundles):
    prefixes = tuple(str(bundle.resolve()) + "/Contents/" for bundle in bundles)
    processes = subprocess.check_output(["/bin/ps", "-axo", "pid=,comm="], text=True)
    for line in processes.splitlines():
        fields = line.strip().split(None, 1)
        if len(fields) == 2 and os.path.realpath(fields[1]).startswith(prefixes):
            raise SystemExit("Quit Devline and let its active CLI jobs finish before installing.")


directory = os.open(applications, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
try:
    fcntl.flock(directory, fcntl.LOCK_EX)
    replacing = installed.exists() or installed.is_symlink()
    if replacing and bundle_id(installed) != "io.devkit.desktop":
        raise SystemExit("A different app or symbolic link already occupies the install location.")
    migrating = not replacing and bundle_id(legacy) == "io.devkit.desktop"
    bundles = (installed, legacy) if migrating else (installed,)
    require_stopped(*bundles)
    mode = RENAME_SWAP if replacing else RENAME_EXCL
    rename(candidate, installed, mode)
    try:
        os.utime(installed)
        subprocess.run([registration, "-f", installed], check=True)
        if migrating:
            require_stopped(legacy)
            rename(legacy, retired, RENAME_EXCL)
    except BaseException:
        try:
            if retired.exists():
                rename(retired, legacy, RENAME_EXCL)
            rename(installed, candidate, mode)
        except OSError:
            (candidate.parent / ".preserve").touch()
            raise
        raise
finally:
    os.close(directory)
PY

INSTALLED_APP="$APPLICATIONS/Devline.app"
LEGACY_APP="$APPLICATIONS/DevKit.app"
# Retarget only a CLI link created by the previous app, preserving unrelated commands.
LEGACY_CLI="$HOME/.local/bin/devkit"
if [ -L "$LEGACY_CLI" ] && [ "$(readlink "$LEGACY_CLI")" = "$LEGACY_APP/Contents/MacOS/devkit" ]; then
    ln -sfn "$INSTALLED_APP/Contents/MacOS/devkit" "$LEGACY_CLI"
fi
printf 'Installed %s\n' "$INSTALLED_APP"
