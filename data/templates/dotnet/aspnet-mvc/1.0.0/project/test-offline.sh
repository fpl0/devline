#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_GENERATE_ASPNET_CERTIFICATE=false
export DOTNET_EnableDiagnostics=0 DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1
"$tool" restore TaskBoard.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
"$tool" build TaskBoard.csproj --no-restore -p:UseSharedCompilation=false
printf '%s\n' 'MVC offline restore and Razor build passed. Start the local services to exercise sign-in and delivery.'
