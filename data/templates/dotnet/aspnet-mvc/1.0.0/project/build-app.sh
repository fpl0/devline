#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_USE_MSBUILD_SERVER=0
"$tool" restore TaskBoard.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
exec "$tool" build TaskBoard.csproj --no-restore -m:1 -p:UseSharedCompilation=false
