#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true
"$tool" restore JsonCli.csproj --locked-mode --configfile NuGet.Config -p:NuGetAudit=false
"$tool" build JsonCli.csproj --no-restore -p:UseSharedCompilation=false
app=bin/Debug/net10.0/json-cli.dll
test "$("$tool" "$app" get /books/0/title examples/catalog.json)" = '"The Left Hand of Darkness"'
test "$(printf '%s' '{"a/b":{"~name":42}}' | "$tool" "$app" get /a~1b/~0name -)" = 42
test "$(printf '%s' '{}' | "$tool" "$app" format -)" = '{}'
if printf '%s' '{' | "$tool" "$app" format - >/dev/null 2>&1; then
    printf '%s\n' 'Invalid JSON must fail.' >&2
    exit 1
fi
if "$tool" "$app" get /missing examples/catalog.json >/dev/null 2>&1; then
    printf '%s\n' 'A missing pointer must fail.' >&2
    exit 1
fi
printf '%s\n' 'JSON CLI smoke passed.'
