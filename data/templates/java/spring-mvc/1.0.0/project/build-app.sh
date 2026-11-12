#!/bin/sh
set -eu
cd "$(dirname "$0")"
repository=${DEVLINE_MAVEN_REPOSITORY:-$PWD/maven-repository}
[ -d "$repository" ] || repository=$PWD/.devline/packages/maven-repository
export MAVEN_OPTS="${MAVEN_OPTS:-} -XX:ActiveProcessorCount=2"
exec "${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-mvn}}" -o -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$repository" -DskipTests package
