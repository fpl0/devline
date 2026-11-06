#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-dotnet}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_GENERATE_ASPNET_CERTIFICATE=false
"$tool" restore --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
exec "$tool" run --no-restore --no-launch-profile -- --smoke-test
