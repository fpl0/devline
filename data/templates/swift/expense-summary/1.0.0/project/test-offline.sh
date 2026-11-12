#!/bin/sh
set -eu
cd "$(dirname "$0")"
scratch=$(mktemp -d "${TMPDIR:-/tmp}/devline-ledger.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
trap 'exit 130' HUP INT TERM
export DEVLINE_SWIFT_SCRATCH="$scratch/swift"
./scripts/swift-local.sh test --disable-xctest
binary=$(./scripts/swift-local.sh build --show-bin-path)/ExpenseSummary
"$binary" --ledger "$scratch/ledger.json" import examples/expenses.json
"$binary" --ledger "$scratch/ledger.json" import examples/expenses.json | grep -q 'Imported 0 of 3'
"$binary" --ledger "$scratch/ledger.json" budget Food 50
"$binary" --ledger "$scratch/ledger.json" report 2026-11 | grep -q 'Total: 30.75 EUR'
"$binary" --ledger "$scratch/ledger.json" export > "$scratch/before.json"
if "$binary" --ledger "$scratch/ledger.json" add 2026-02-30 Food 3 Invalid > "$scratch/error" 2>&1; then
    echo 'Invalid date was accepted' >&2; exit 1
fi
"$binary" --ledger "$scratch/ledger.json" export > "$scratch/after.json"
cmp "$scratch/before.json" "$scratch/after.json"
"$binary" --ledger "$scratch/ledger.json" remove nov-train
"$binary" --ledger "$scratch/ledger.json" list | grep -q 'Total: 13.00 EUR'
printf '%s\n' 'Expense Ledger domain and command-line workflows passed.'
