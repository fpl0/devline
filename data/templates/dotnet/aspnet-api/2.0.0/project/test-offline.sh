#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_CLI_USE_MSBUILD_SERVER=0 MSBUILDDISABLENODEREUSE=1
"$tool" restore OfflineApi.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
"$tool" build OfflineApi.csproj --no-restore -m:1 -p:UseSharedCompilation=false
python3 test_http.py dotnet
