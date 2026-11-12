#!/bin/sh
# Verify and unpack the bundled browser into a temporary directory for one run.
set -eu
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-node}}
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || { echo 'This browser release targets macOS Apple Silicon.' >&2; exit 1; }
manifest=BROWSER.json
archive=browser-archive
if [ -f .devline/BROWSER.json ]; then manifest=.devline/BROWSER.json; archive=.devline/packages/browser-archive; fi
revision=$("$tool" --input-type=module -e 'import{readFileSync}from"node:fs";const b=JSON.parse(readFileSync(process.argv[1]));const actual=process.argv[2].split(".").map(Number),required=b.minimum_macos.split(".").map(Number);const difference=required.map((n,i)=>(actual[i]||0)-n).find(n=>n!==0)||0;if(difference<0)throw Error(`The bundled WebKit requires macOS ${b.minimum_macos} or later.`);if(!/^\d+$/.test(b.revision))throw Error("Invalid browser revision");console.log(b.revision)' "$manifest" "$(/usr/bin/sw_vers -productVersion)")
(cd "$archive" && /usr/bin/shasum -a 256 -c SHA256SUMS)
browser_dir=$(mktemp -d "${TMPDIR:-/tmp}/site-audit-browser.XXXXXX")
trap 'rm -rf "$browser_dir"' EXIT HUP INT TERM
cat "$archive"/browser-*.part > "$browser_dir/webkit.zip"
/usr/bin/ditto -x -k "$browser_dir/webkit.zip" "$browser_dir/webkit-$revision"
rm "$browser_dir/webkit.zip"
export PLAYWRIGHT_BROWSERS_PATH="$browser_dir"
"$tool" "$@"
