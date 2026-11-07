#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-node}
export PATH="$(dirname "$tool"):$PATH" CI=true
"$tool" node_modules/typescript/bin/tsc --noEmit
"$tool" node_modules/vitest/vitest.mjs run --environment happy-dom
"$tool" node_modules/vite/bin/vite.js build
