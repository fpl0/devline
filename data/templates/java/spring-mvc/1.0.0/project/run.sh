#!/bin/sh
set -eu
cd "$(dirname "$0")"
repository=${DEVLINE_MAVEN_REPOSITORY:-$PWD/maven-repository}
[ -d "$repository" ] || repository=$PWD/.devline/packages/maven-repository
"${DEVKIT_TOOLCHAIN:-mvn}" -q -o "-Dmaven.repo.local=$repository" -DskipTests package
if [ -n "${JAVA_HOME:-}" ]; then java="$JAVA_HOME/bin/java"; else java=java; fi
base=${DEVLINE_APP_URL:-${ASPNETCORE_URLS:-http://127.0.0.1:8081}}
port=${base##*:}
export TASK_BOARD_BASE_URL="http://127.0.0.1:$port"
if [ -z "${DEVLINE_KEYCLOAK_URL:-}" ] || [ -z "${DEVLINE_KEYCLOAK_PASSWORD:-}" ]; then
    printf '%s\n' 'Configure PostgreSQL, Redis, RabbitMQ and Keycloak in this project first.' >&2
    exit 2
fi
exec "$java" -jar target/task-board.jar "--server.port=$port"
