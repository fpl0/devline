#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then set -- examples/expenses.json; fi
exec ./scripts/swift-local.sh run ExpenseSummary "$@"
