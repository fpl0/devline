#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-ruby}
result=$(printf '%s' '{ "status": "ok" }' | "$tool" main.rb)
test "$result" = '{"status":"ok"}'
echo 'Ruby JSON CLI test passed'
