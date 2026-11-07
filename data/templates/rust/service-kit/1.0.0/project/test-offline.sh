#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec "${DEVKIT_TOOLCHAIN:-cargo}" test --frozen
