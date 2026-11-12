#!/bin/sh
set -eu
cd "$(dirname "$0")"
repository=${DEVLINE_MAVEN_REPOSITORY:-$PWD/maven-repository}
[ -d "$repository" ] || repository=$PWD/.devline/packages/maven-repository
export MAVEN_OPTS="${MAVEN_OPTS:-} -XX:ActiveProcessorCount=2"
"${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-mvn}}" -q -o "-Dmaven.repo.local=$repository" -DskipTests package
if [ -n "${JAVA_HOME:-}" ]; then java="$JAVA_HOME/bin/java"; else java=java; fi
base=${DEVLINE_APP_URL:-http://127.0.0.1:8081}
exec "$java" -XX:ActiveProcessorCount=2 -jar target/task-board.jar "--server.port=${base##*:}"
