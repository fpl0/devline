#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-node}
export PATH="$(dirname "$tool"):$PATH" CI=true
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || { echo 'This browser release targets macOS Apple Silicon.' >&2; exit 1; }
revision=$("$tool" --input-type=module - "$(/usr/bin/sw_vers -productVersion)" <<'JS'
import { readFileSync } from 'node:fs';
const { revision, minimum_macos } = JSON.parse(readFileSync('BROWSER.json'));
const current = process.argv[2].split('.').map(Number);
const required = minimum_macos.split('.').map(Number);
const difference = required.map((version, index) => (current[index] || 0) - version).find(value => value !== 0) || 0;
if (difference < 0) {
  console.error(`The bundled Playwright WebKit browser requires macOS ${minimum_macos} or later.`);
  process.exit(1);
}
if (!/^\d+$/.test(revision)) throw new Error('Invalid browser archive revision.');
console.log(revision);
JS
)
(cd browser-archive && /usr/bin/shasum -a 256 -c SHA256SUMS)
browser_dir=$(mktemp -d "${TMPDIR:-/tmp}/devline-browser-test.XXXXXX")
trap 'rm -rf "$browser_dir"' EXIT HUP INT TERM
cat browser-archive/browser-*.part > "$browser_dir/webkit.zip"
/usr/bin/ditto -x -k "$browser_dir/webkit.zip" "$browser_dir/webkit-$revision"
rm "$browser_dir/webkit.zip"
export PLAYWRIGHT_BROWSERS_PATH="$browser_dir"
"$tool" node_modules/@playwright/test/cli.js test "$@"
