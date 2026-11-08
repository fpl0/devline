#!/bin/sh
set -eu
cd "$(dirname "$0")"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-expenses.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
export DEVLINE_SWIFT_SCRATCH="$scratch/swift"
output=$(./scripts/swift-local.sh run ExpenseSummary examples/expenses.json)
test "$output" = 'Food: 13.00
Travel: 17.75
Total: 30.75'
printf '%s\n' '[{"category":"Food","cents":-100}]' >"$scratch/invalid.json"
if ./scripts/swift-local.sh run ExpenseSummary "$scratch/invalid.json" >"$scratch/error" 2>&1; then
    echo 'Expected a negative expense to fail' >&2
    exit 1
fi
grep -q 'amount greater than zero' "$scratch/error"
