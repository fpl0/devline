#!/bin/sh
set -eu
cd "$(dirname "$0")"
repository=${DEVLINE_MAVEN_REPOSITORY:-$PWD/maven-repository}
[ -d "$repository" ] || repository=$PWD/.devline/packages/maven-repository
"${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-mvn}}" -q -o "-Dmaven.repo.local=$repository" -DskipTests package
if [ -n "${JAVA_HOME:-}" ]; then java="$JAVA_HOME/bin/java"; else java=java; fi
if [ "$#" -eq 0 ]; then set -- --invoices fixtures/invoices.csv --payments fixtures/payments.csv --output reports/reconciliation.json; fi
exec "$java" -jar target/json-tool.jar "$@"
