#!/bin/sh
set -eu
cd "$(dirname "$0")"
export MAVEN_OPTS="${MAVEN_OPTS:-} -XX:ActiveProcessorCount=2"
"${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-mvn}}" -o -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$PWD/maven-repository" verify
python3 test_http.py java
