#!/bin/sh
# This starter's dependencies are provided by the supported Ruby toolchain.
set -eu
test -f "${1:?project folder}/main.rb"
