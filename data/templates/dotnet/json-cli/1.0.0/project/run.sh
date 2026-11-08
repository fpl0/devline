#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
if [ "$#" -eq 0 ]; then set -- format examples/catalog.json; fi
exec "$tool" run --project JsonCli.csproj -p:RestoreLockedMode=true -p:NuGetAudit=false -- "$@"
