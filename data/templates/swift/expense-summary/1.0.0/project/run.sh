#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then
    ./scripts/swift-local.sh run ExpenseSummary --ledger .data/demo.json import examples/expenses.json
    ./scripts/swift-local.sh run ExpenseSummary --ledger .data/demo.json budget Food 50
    exec ./scripts/swift-local.sh run ExpenseSummary --ledger .data/demo.json report 2026-11
fi
exec ./scripts/swift-local.sh run ExpenseSummary "$@"
