#!/bin/sh
set -eu
cd "$(dirname "$0")"
runner=./run.sh
[ -f "$runner" ] || runner=./.devline/scripts/run.sh
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_CLI_USE_MSBUILD_SERVER=0
"$runner" --self-test
"$runner"
python3 - <<'CHECK'
import json
r=json.load(open('reports/reconciliation.json'))
assert [i['status'] for i in r['invoices']]==['paid','partial','unpaid','overpaid']
assert len(r['unmatched'])==1
print('Packaged reconciliation workflow passed.')
CHECK
