#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVKIT_TOOLCHAIN:-dotnet}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true DOTNET_GENERATE_ASPNET_CERTIFICATE=false
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-report.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
mkdir "$scratch/inputs"
printf 'one\n' > "$scratch/inputs/one.txt"
printf 'two\n' > "$scratch/inputs/two.txt"
"$tool" restore ReportWorker.csproj --configfile NuGet.Config -p:NuGetAudit=false
Reports__InputDirectory="$scratch/inputs" Reports__OutputFile="$scratch/latest.json" Reports__RunOnce=true \
  "$tool" run --project ReportWorker.csproj --no-restore --no-launch-profile
grep -Fqx '{"Files":2,"Bytes":8}' "$scratch/latest.json"
printf 'Hosted worker report verified.\n'
