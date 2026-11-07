#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-dotnet}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_GENERATE_ASPNET_CERTIFICATE=false
export DOTNET_EnableDiagnostics=0 DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1
export ASPNETCORE_URLS=${ASPNETCORE_URLS:-http://127.0.0.1:5080}
"$tool" restore OfflineApi.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false -p:UseSharedCompilation=false
exec "$tool" run --project OfflineApi.csproj --no-restore --no-launch-profile -p:UseSharedCompilation=false
