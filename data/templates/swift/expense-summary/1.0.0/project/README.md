# Expense Ledger

A local expense journal for recording receipts, importing transactions, and comparing monthly spending with recurring category budgets. Everything stays in a JSON file on your Mac. Amounts use integer cents throughout, so reports cannot accumulate floating-point rounding errors.

## Run a complete workflow

Use Swift 6 and Xcode 16 or later. The project has no external dependencies.

```sh
./run.sh                         # import the sample into .data/demo.json and print a report
./run.sh import examples/expenses.json
./run.sh budget Food 50.00
./run.sh add 2026-11-12 Food 12.50 'Lunch with the team' lunch-2026-11-12
./run.sh list 2026-11
./run.sh report 2026-11
./run.sh remove lunch-2026-11-12
./run.sh export > expenses-backup.json
./test-offline.sh
```

In a generated Devline project, `./devline.sh run` runs the sample. To pass custom arguments, use `./devline.sh exec /bin/sh .devline/scripts/run.sh report 2026-11`. To distribute the CLI, build with `./.devline/scripts/swift-local.sh build --configuration release`; `--show-bin-path` locates the executable. In the source project the helper lives under `scripts/`.

The default journal is `.data/expenses.json`. Select another with `./run.sh --ledger /path/to/journal.json COMMAND`. The default currency is EUR; this is a single-currency expense journal, with no exchange-rate conversion, refund, or accounting functionality. Budgets recur each month; changing a budget changes comparisons for every month. Exports contain the complete schema, expenses, and budgets. Restore a backup by keeping the original file and copying the backup to a new ledger path, then run `list` against that path to validate it.

## Import format and failure behavior

An import is a JSON array of objects containing `reference`, `date`, `category`, `description`, and `amount`. See `examples/expenses.json`. Dates must be real calendar dates in `YYYY-MM-DD` form. Amounts are positive decimal **strings** with at most two fractional digits.

References are stable external identifiers. Reimporting an identical transaction is harmless; reusing a reference for different data rejects the entire batch. A malformed row, invalid date, conflicting reference, or total overflow leaves the journal unchanged. A process lock covers read/modify/write, and file replacement is atomic. Corrupt files and unsupported schema versions produce errors rather than being replaced with an empty journal. Failed commands return a nonzero exit code and send errors to stderr; reports go to stdout.

## Code to explore

- `ExpenseLedger/Ledger.swift`: value types, exact money parsing, validation, idempotent batch import, and monthly reports.
- `ExpenseLedger/LedgerFile.swift`: isolated file transactions, locking, and atomic persistence.
- `ExpenseSummary/ExpenseSummary.swift`: command parsing and presentation, separate from domain logic.
- `Tests/ExpenseLedgerTests`: failure atomicity, round trips, stale/bad input, budgets, date validation, and numeric limits. The shell test also exercises the compiled CLI across separate processes.

A useful next extension is per-month budgets: introduce a versioned migration and key budgets by month and category, keeping the existing exact-money invariants.
