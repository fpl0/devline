#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/devline-install-test.XXXXXX")"
running=""
cleanup() {
    if [[ -n "$running" ]]; then
        kill "$running" 2>/dev/null || true
        wait "$running" 2>/dev/null || true
    fi
    rm -rf "$SCRATCH"
}
trap cleanup EXIT
mkdir -p "$SCRATCH/repo/scripts"
installer="$SCRATCH/repo/scripts/install.sh"
# Refuse real Launch Services registration in the disposable home and allow
# explicit post-swap failure and success to be tested.
mock_registration() {
    /usr/bin/sed "s|/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister|$1|" \
        "$ROOT/scripts/install.sh" > "$installer"
}
mock_registration /usr/bin/false
/usr/bin/clang -arch arm64 -mmacosx-version-min=14.0 -x c - -o "$SCRATCH/executable" <<'C'
#include <unistd.h>
int main(int argc, char **argv) {
    if (argc > 1) sleep(30);
    return 0;
}
C
bundle() {
    local app="$1" id="$2" marker="$3"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    cp "$SCRATCH/executable" "$app/Contents/MacOS/DevlineApp"
    cp "$SCRATCH/executable" "$app/Contents/MacOS/devline"
    ln -s devline "$app/Contents/MacOS/devkit"
    cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>DevlineApp</string>
<key>CFBundleIdentifier</key><string>$id</string>
<key>CFBundleDisplayName</key><string>Devline</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>
PLIST
    printf '%s\n' "$marker" > "$app/Contents/Resources/marker"
    /usr/bin/codesign --force --sign - "$app"
}
archive() {
    local app="$1"
    rm -f "$SCRATCH/Devline-macos-arm64.zip"
    /usr/bin/ditto --norsrc -c -k --keepParent "$app" "$SCRATCH/Devline-macos-arm64.zip"
}
expect_failure() {
    local test_user_root="$1" diagnostic="$2"
    if env HOME="$test_user_root" /bin/bash "$installer" > "$SCRATCH/install.log" 2>&1; then
        printf 'Installer unexpectedly succeeded for %s\n' "$test_user_root" >&2
        exit 1
    fi
    grep -Fq "$diagnostic" "$SCRATCH/install.log" || { cat "$SCRATCH/install.log" >&2; exit 1; }
}
start_process() {
    "$1/Contents/MacOS/devline" --hold &
    running=$!
    /bin/ps -p "$running" -o pid= >/dev/null
}
stop_process() {
    kill "$running"
    wait "$running" 2>/dev/null || true
    running=""
}

candidate="$SCRATCH/candidate/Devline.app"
bundle "$candidate" io.devkit.desktop new
archive "$candidate"
legacy_user="$SCRATCH/Legacy Test User"
legacy="$legacy_user/Applications/DevKit.app"
bundle "$legacy" io.devkit.desktop legacy
start_process "$legacy"
expect_failure "$legacy_user" "Quit Devline and let its active CLI jobs finish"
[[ "$(cat "$legacy/Contents/Resources/marker")" == legacy && ! -e "$legacy_user/Applications/Devline.app" ]]
stop_process

current_user="$SCRATCH/Current Test User"
current="$current_user/Applications/Devline.app"
bundle "$current" io.devkit.desktop current
start_process "$current"
expect_failure "$current_user" "Quit Devline and let its active CLI jobs finish"
[[ "$(cat "$current/Contents/Resources/marker")" == current ]]
stop_process
expect_failure "$current_user" "returned non-zero exit status"
[[ "$(cat "$current/Contents/Resources/marker")" == current ]]
/usr/bin/codesign --verify --deep --strict "$current"

wrong="$SCRATCH/foreign/Devline.app"
bundle "$wrong" example.foreign.application foreign
archive "$wrong"
wrong_user="$SCRATCH/Wrong Test User"
mkdir "$wrong_user"
expect_failure "$wrong_user" "The ZIP does not contain the current Devline app"
[[ ! -e "$wrong_user/Applications/Devline.app" ]]

archive "$candidate"
/usr/bin/ditto "$wrong" "$wrong_user/Applications/Devline.app"
expect_failure "$wrong_user" "A different app or symbolic link already occupies the install location"
[[ "$(cat "$wrong_user/Applications/Devline.app/Contents/Resources/marker")" == foreign ]]

mock_registration /usr/bin/true
mkdir -p "$legacy_user/.local/bin"
ln -s "$legacy/Contents/MacOS/devkit" "$legacy_user/.local/bin/devkit"
env HOME="$legacy_user" /bin/bash "$installer" > "$SCRATCH/install.log" 2>&1
[[ ! -e "$legacy" && "$(cat "$legacy_user/Applications/Devline.app/Contents/Resources/marker")" == new ]]
[[ "$(readlink "$legacy_user/.local/bin/devkit")" == "$legacy_user/Applications/Devline.app/Contents/MacOS/devkit" ]]
printf 'Installer guard, incoming identity, rollback and legacy migration passed.\n'
