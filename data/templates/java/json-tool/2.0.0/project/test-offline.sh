#!/bin/sh
set -eu
cd "$(dirname "$0")"
runner=./run.sh
[ -f "$runner" ] || runner=./.devline/scripts/run.sh
export MAVEN_OPTS="${MAVEN_OPTS:-} -XX:ActiveProcessorCount=2"
"${DEVLINE_TOOLCHAIN:-${DEVKIT_TOOLCHAIN:-mvn}}" -o -B -ntp -s settings-offline.xml "-Dmaven.repo.local=$PWD/maven-repository" verify
"$runner"
python3 - <<'CHECK'
import json
r=json.load(open('reports/reconciliation.json'))
assert [i['status'] for i in r['invoices']]==['paid','partial','unpaid','overpaid']
assert len(r['unmatched'])==1
print('Reconciliation domain and packaged CLI checks passed.')
CHECK
