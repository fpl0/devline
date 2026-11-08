#!/bin/sh
set -eu
cd "$(dirname "$0")"
"${DEVKIT_TOOLCHAIN:-mvn}" -o -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$PWD/maven-repository" verify
if [ -n "${JAVA_HOME:-}" ]; then java="$JAVA_HOME/bin/java"; else java=java; fi
"$java" -jar target/json-tool.jar inspect fixtures/example.json | grep -Fqx '{"type":"object","entries":2}'
printf '%s\n' '{ "ready": true }' | "$java" -jar target/json-tool.jar format --compact - | grep -Fqx '{"ready":true}'
