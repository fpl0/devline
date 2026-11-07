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
cache="$project/.devline-offline/dotnet/packages"
merge "$artifacts/packages" "$cache"
export DOTNET_CLI_HOME="$HOME" NUGET_PACKAGES="$cache" NUGET_HTTP_CACHE_PATH="$TMPDIR/nuget-http"
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_GENERATE_ASPNET_CERTIFICATE=false DOTNET_EnableDiagnostics=0
export MSBUILDDISABLENODEREUSE=1 UseSharedCompilation=false NuGetAudit=false
add_package() { "$tool" add "$entry" package "$package_name" --version "[$package_version]" --no-restore; }
packages
feed="$TMPDIR/client-feed"
mkdir "$feed"
for source in "$artifacts/feed" "$project/nuget-packages" "$cache"; do
    [ -d "$source" ] || continue
    find "$source" -type f -name '*.nupkg' -exec /bin/cp -n {} "$feed/" \;
done
if ! "$tool" restore "$entry" --use-lock-file --force-evaluate --source "$feed" -p:NuGetAudit=false; then
    [ "$offline" != true ] || exit 1
    "$tool" restore "$entry" --use-lock-file --force-evaluate --source "$feed" --source https://api.nuget.org/v3/index.json -p:NuGetAudit=false
fi
# Keep a portable local feed for ordinary dotnet restore too.
mkdir -p nuget-packages
find "$cache" -type f -name '*.nupkg' -exec /bin/cp -n {} "$project/nuget-packages/" \;
if [ ! -f NuGet.Config ]; then
    printf '%s\n' '<configuration><packageSources><clear /><add key="Devline local packages" value="nuget-packages" /></packageSources></configuration>' > NuGet.Config
fi
exec /bin/sh "$catalog/project-adapters/dotnet.sh" prepare "$project" "$project/.devline-offline" "$tool"
