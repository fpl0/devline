#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-dotnet}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_GENERATE_ASPNET_CERTIFICATE=false
"$tool" restore tests/OfflineApi.Tests.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
"$tool" run --project OfflineApi.csproj --no-restore --no-launch-profile -- --smoke-test
exec "$tool" test tests/OfflineApi.Tests.csproj --no-restore
