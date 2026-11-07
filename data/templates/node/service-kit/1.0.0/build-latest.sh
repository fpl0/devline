#!/bin/sh
set -eu
project=${1:?project folder}
cd "$project"
tool=${DEVKIT_TOOLCHAIN:-node}
export PATH="$(dirname "$tool"):$PATH"
"$tool" --input-type=module -e 'import{readFileSync}from"node:fs";import{execFileSync}from"node:child_process";const specs=JSON.parse(readFileSync("dependency-specs.json"));execFileSync("npm",["install","--save-exact","--ignore-scripts","--bin-links=false","--no-audit","--no-fund",...specs],{stdio:"inherit"});'
if [ -f prepare-browser.mjs ]; then "$tool" prepare-browser.mjs; fi
