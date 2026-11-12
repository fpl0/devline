#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec "${DEVKIT_TOOLCHAIN:-python3}" -B -m unittest discover -s tests -v
