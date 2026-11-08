#!/bin/sh
set -eu
action=${1:?prepare or verify}
project=${2:?staged project}
cache=${3:?managed dependency directory}/dotnet
tool=${4:?dotnet executable}
shift 4
cd "$project"
export PATH="$(dirname "$tool"):$PATH"
export DEVKIT_TOOLCHAIN="$tool" DOTNET_CLI_TELEMETRY_OPTOUT=1
export DOTNET_GENERATE_ASPNET_CERTIFICATE=false DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true
export DOTNET_EnableDiagnostics=0 DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1 UseSharedCompilation=false
# Cloud-backed folders can stall native file notifications during ASP.NET startup.
# Polling keeps configuration reload available; an explicit user setting wins.
export DOTNET_USE_POLLING_FILE_WATCHER="${DOTNET_USE_POLLING_FILE_WATCHER:-1}"
starter_test=./.devline/scripts/test-offline.sh
[ -f "$starter_test" ] || starter_test=./test-offline.sh
starter() {
    [ -x "$starter_test" ] && { [ -f .devline/origin.json ] || [ -f .devkit-origin.json ]; }
}
if [ "$action" = run ]; then
    [ "$#" -gt 0 ] || { printf '%s\n' 'Specify a .NET project command.' >&2; exit 2; }
    if [ ! -d "$cache/packages" ]; then
        if starter; then
            export DOTNET_CLI_HOME="$project/obj/devline-home" NUGET_PACKAGES="$project/obj/devline-nuget"
            export XDG_DATA_HOME="$DOTNET_CLI_HOME/.local/share" NUGET_HTTP_CACHE_PATH="$project/obj/devline-http"
            export NuGetAudit=false RestoreConfigFile="$project/NuGet.Config"
            if [ -f packages.lock.json ]; then export RestoreLockedMode=true; fi
            feed="$project/.devline/packages/nuget-packages"
            [ -d "$feed" ] || feed="$project/nuget-packages"
            if [ -d "$feed" ]; then export RestoreSources="$feed"; fi
            if [ "$1" = dotnet ]; then shift; exec "$tool" "$@"; fi
            exec "$@"
        fi
        printf '%s\n' 'Prepare this project before running with a local NuGet cache.' >&2
        exit 1
    fi
    scratch=$(mktemp -d)
    trap 'rm -rf "$scratch"' EXIT
    trap 'exit 130' HUP INT TERM
    mkdir "$scratch/home"
    /bin/cp -cR "$cache/packages" "$scratch/packages"
    export DOTNET_CLI_HOME="$scratch/home" NUGET_PACKAGES="$scratch/packages"
    export XDG_DATA_HOME="$scratch/home/.local/share"
    export NUGET_HTTP_CACHE_PATH="$scratch/http-cache" NuGetAudit=false RestoreLockedMode=true
    export RestoreConfigFile="$scratch/NuGet.Config"
    printf '%s\n' '<configuration><packageSources><clear /></packageSources></configuration>' > "$RestoreConfigFile"
    if [ "$1" = dotnet ]; then shift; "$tool" "$@"; else "$@"; fi
    exit $?
fi

# A solution can describe several projects. Without one, require one root project.
set -- ./*.slnx ./*.sln
entry=
for file do
    [ -f "$file" ] || continue
    if [ -n "$entry" ]; then
        printf '%s\n' 'Choose a project directory with one .NET solution.' >&2
        exit 1
    fi
    entry=$file
done
if [ -z "$entry" ]; then
    set -- ./*.csproj ./*.fsproj ./*.vbproj
    for file do
        [ -f "$file" ] || continue
        if [ -n "$entry" ]; then
            printf '%s\n' 'Use a solution to prepare multiple .NET projects together.' >&2
            exit 1
        fi
        entry=$file
    done
fi
if [ -z "$entry" ]; then
    printf '%s\n' 'Choose the directory containing the .NET solution or project file.' >&2
    exit 1
fi

export DOTNET_CLI_HOME="$HOME"
export XDG_DATA_HOME="$HOME/.local/share"
export NUGET_HTTP_CACHE_PATH="${TMPDIR:?}/nuget-http"
if [ "$action" = verify ]; then
    if [ ! -d "$cache/packages" ]; then
        if starter; then exec "$starter_test"; fi
        printf '%s\n' 'Prepare this project while connected to archive its NuGet packages.' >&2
        exit 1
    fi
    export NUGET_PACKAGES="$TMPDIR/nuget-packages"
    /bin/cp -cR "$cache/packages" "$NUGET_PACKAGES"
else
    export NUGET_PACKAGES="$cache/packages"
    mkdir -p "$NUGET_PACKAGES"
fi

restore_locked() {
    # Locked restore may generate a new lock for a project without one. Refuse
    # such a graph so the existing project remains reproducible as authored.
    before=$(mktemp "$TMPDIR/dotnet-lock-before.XXXXXX")
    after=$(mktemp "$TMPDIR/dotnet-lock-after.XXXXXX")
    trap 'rm -f "$before" "$after"' EXIT
    trap 'exit 130' HUP INT TERM
    locks() {
        find . \( -name .git -o -name .devline -o -name .devline-offline -o -name bin -o -name obj \) -prune -o \
            -type f -name packages.lock.json -exec /usr/bin/shasum -a 256 {} + | LC_ALL=C sort
    }
    locks > "$before"
    if [ ! -s "$before" ]; then
        printf '%s\n' 'Generate packages.lock.json with dotnet restore --use-lock-file, then prepare again.' >&2
        exit 1
    fi
    "$tool" restore "$entry" --use-lock-file --locked-mode -p:NuGetAudit=false "$@"
    locks > "$after"
    if ! cmp -s "$before" "$after"; then
        printf '%s\n' 'Every referenced .NET project must already have an up-to-date packages.lock.json.' >&2
        exit 1
    fi
}

case "$action" in
    prepare)
        if [ "${DEVLINE_OFFLINE:-false}" = true ]; then
            config=$(mktemp "$TMPDIR/dotnet-offline-config.XXXXXX")
            printf '%s\n' '<configuration><packageSources><clear /></packageSources></configuration>' > "$config"
            restore_locked --configfile "$config"
            rm -f "$config"
        else
            restore_locked
        fi
        ;;
    verify)
        config=$(mktemp "$TMPDIR/dotnet-config.XXXXXX")
        printf '%s\n' '<configuration><packageSources><clear /></packageSources></configuration>' > "$config"
        restore_locked --configfile "$config"
        "$tool" build "$entry" --no-restore
        "$tool" test "$entry" --no-build --no-restore
        rm -f "$config"
        ;;
    *) printf '%s\n' 'Expected prepare or verify.' >&2; exit 2 ;;
esac
