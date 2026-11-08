#!/bin/sh
set -eu
action=${1:?prepare or verify}
project=${2:?staged project}
cache=${3:?managed dependency directory}/java
tool=${4:?Maven executable}
shift 4
cd "$project"
export PATH="$(dirname "$tool"):$PATH"
export DEVKIT_TOOLCHAIN="$tool"
starter_test=./.devline/scripts/test-offline.sh
[ -f "$starter_test" ] || starter_test=./test-offline.sh
if [ "$action" = run ]; then
    [ "$#" -gt 0 ] || { printf '%s\n' 'Specify a Maven project command.' >&2; exit 2; }
    repository="$cache/repository"
    [ -d "$repository" ] || repository="$project/.devline/packages/maven-repository"
    [ -d "$repository" ] || repository="$project/maven-repository"
    if [ ! -d "$repository" ]; then
        printf '%s\n' 'Prepare this project before running with a local Maven repository.' >&2
        exit 1
    fi
    scratch=$(mktemp -d)
    trap 'rm -rf "$scratch"' EXIT
    trap 'exit 130' HUP INT TERM
    /bin/cp -cR "$repository" "$scratch/repository"
    export DEVLINE_MAVEN_REPOSITORY="$scratch/repository" MAVEN_ARGS="-o"
    export JDK_JAVA_OPTIONS="${JDK_JAVA_OPTIONS:-} -Dmaven.repo.local=\"$DEVLINE_MAVEN_REPOSITORY\""
    if [ "$1" = mvn ]; then
        shift
        "$tool" -o "-Dmaven.repo.local=$DEVLINE_MAVEN_REPOSITORY" "$@"
    else
        "$@"
    fi
    exit $?
fi
if [ ! -f pom.xml ]; then
    printf '%s\n' 'Choose the Maven project directory containing pom.xml.' >&2
    exit 1
fi
# The Java launcher parses quotes here. Maven expands MAVEN_OPTS with shell
# word splitting, which would break staged project paths containing spaces.
export JDK_JAVA_OPTIONS="${JDK_JAVA_OPTIONS:-} -Duser.home=\"$HOME\" -Djava.io.tmpdir=\"${TMPDIR:?}\""
case "$action" in
    prepare)
        mkdir -p "$cache/repository"
        if [ "${DEVLINE_OFFLINE:-0}" = 1 ]; then
            "$tool" -o -B -ntp "-Dmaven.repo.local=$cache/repository" clean verify
        else
            "$tool" -B -ntp "-Dmaven.repo.local=$cache/repository" clean verify
        fi
        ;;
    verify)
        if [ ! -d "$cache/repository" ]; then
            if [ -x "$starter_test" ] && { [ -f .devline/origin.json ] || [ -f .devkit-origin.json ]; }; then
                exec "$starter_test"
            fi
            printf '%s\n' 'Prepare this project while connected to archive its Maven dependencies.' >&2
            exit 1
        fi
        /bin/cp -cR "$cache/repository" "${TMPDIR:?}/maven-repository"
        "$tool" -o -B -ntp "-Dmaven.repo.local=$TMPDIR/maven-repository" clean verify
        ;;
    *) printf '%s\n' 'Expected prepare or verify.' >&2; exit 2 ;;
esac
