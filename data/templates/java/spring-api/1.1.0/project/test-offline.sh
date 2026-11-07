#!/bin/sh
set -eu
cd "$(dirname "$0")"
exec "${DEVKIT_TOOLCHAIN:-mvn}" -o -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$PWD/maven-repository" verify
