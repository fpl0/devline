#!/bin/sh
set -eu
cd "$(dirname "$0")"
"${DEVKIT_TOOLCHAIN:-mvn}" -o -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$PWD/maven-repository" verify
if [ -n "${JAVA_HOME:-}" ]; then java="$JAVA_HOME/bin/java"; else java=java; fi
"$java" -jar "target/__PROJECT_NAME__-1.0.0-cli.jar" fixtures/example.json | grep -Fqx '{"type":"object","entries":2}'
