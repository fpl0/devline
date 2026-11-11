#!/bin/sh
set -eu
. "$(dirname "$0")/common.sh"
cd "$project"
entry=
for file in ./*.csproj ./*.fsproj ./*.vbproj; do
    [ -f "$file" ] || continue
    [ -z "$entry" ] || { echo 'Choose a directory with one application project before adding client packages.' >&2; exit 1; }
    entry=$file
done
[ -n "$entry" ] || { echo 'Open the application project directory containing its .csproj.' >&2; exit 1; }
cache="$managed/dotnet/packages"
local_feed=nuget-packages
if [ "$managed_relative" = .devline/cache ]; then local_feed=.devline/packages/nuget-packages; fi
merge "$artifacts/packages" "$cache"
export DOTNET_CLI_HOME="$HOME" NUGET_PACKAGES="$cache" NUGET_HTTP_CACHE_PATH="$TMPDIR/nuget-http"
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_GENERATE_ASPNET_CERTIFICATE=false DOTNET_EnableDiagnostics=0
export MSBUILDDISABLENODEREUSE=1 UseSharedCompilation=false NuGetAudit=false
# .NET keeps cross-process locks under /tmp/.dotnet and creates that directory through a
# temporary sibling, which the stage sandbox forbids. Creating it directly is allowed, and
# does nothing once any .NET command has run since the Mac started.
for shared in /private/tmp/.dotnet /private/tmp/.dotnet/shm /private/tmp/.dotnet/lockfiles; do
    [ -d "$shared" ] || /bin/mkdir -m 1777 "$shared" 2>/dev/null || :
done
add_package() { "$tool" add "$entry" package "$package_name" --version "[$package_version]" --no-restore; }
packages
feed="$TMPDIR/client-feed"
mkdir "$feed"
for source in "$artifacts/feed" "$project/$local_feed" "$cache"; do
    [ -d "$source" ] || continue
    find "$source" -type f -name '*.nupkg' -exec /bin/cp -n {} "$feed/" \;
done
set -- --source "$feed"
# The package shelf is read in place as one more local feed: restore copies only the
# packages this project's graph uses into its cache.
if [ -n "$shelf" ] && [ -d "$shelf/feed" ]; then set -- "$@" --source "$shelf/feed"; fi
if ! "$tool" restore "$entry" --use-lock-file --force-evaluate "$@" -p:NuGetAudit=false; then
    [ "$offline" != true ] || exit 1
    "$tool" restore "$entry" --use-lock-file --force-evaluate "$@" --source https://api.nuget.org/v3/index.json -p:NuGetAudit=false
fi
# Keep a portable local feed for ordinary dotnet restore too.
mkdir -p "$local_feed"
find "$cache" -type f -name '*.nupkg' -exec /bin/cp -n {} "$project/$local_feed/" \;
if [ ! -f NuGet.Config ]; then
    printf '<configuration><packageSources><clear /><add key="Devline local packages" value="%s" /></packageSources></configuration>\n' "$local_feed" > NuGet.Config
elif [ "$managed_relative" = .devline/cache ]; then
    # A dependency-free starter begins with an empty source list, written on one line or several.
    case "$(/usr/bin/sed -n '/<packageSources>/,/<\/packageSources>/p' NuGet.Config)" in
        *'<add '*) ;;
        *)
            /usr/bin/sed '/<packageSources>/,/<\/packageSources>/s|<clear />|<clear /><add key="Devline local packages" value=".devline/packages/nuget-packages" />|' NuGet.Config > "$TMPDIR/client-NuGet.Config"
            /bin/cp "$TMPDIR/client-NuGet.Config" NuGet.Config
            ;;
    esac
fi
exec /bin/sh "$catalog/project-adapters/dotnet.sh" prepare "$project" "$managed" "$tool"
