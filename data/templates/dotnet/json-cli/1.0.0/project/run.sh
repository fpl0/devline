#!/bin/sh
set -eu
cd "$(dirname "$0")"
tool=${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-dotnet}}
if [ "$#" -eq 0 ]; then set -- --invoices fixtures/invoices.csv --payments fixtures/payments.csv --output reports/reconciliation.json; fi
exec "$tool" run --project JsonCli.csproj -p:UseSharedCompilation=false -p:RestoreLockedMode=true -p:NuGetAudit=false -- "$@"
