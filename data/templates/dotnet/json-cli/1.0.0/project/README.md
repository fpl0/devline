# Invoice Reconciliation

Reconcile invoice balances against bank payments in a reproducible batch. Multiple payments settle an invoice together; partial, missing and excess payments remain visible. Unknown references and currency mismatches are reported separately. Amounts use exact decimal arithmetic; no floating point or currency conversion is involved.

```sh
./run.sh
dotnet bin/Debug/net10.0/json-cli.dll --invoices fixtures/invoices.csv --payments fixtures/payments.csv --output reports/reconciliation.json
./build-app.sh
./test-offline.sh
```

The sample produces one paid, one partial, one unpaid and one overpaid invoice, plus an unmatched payment. The JSON report includes the input totals by currency, all invoice balances and unmatched payments. Run again after correcting the source CSVs. Output is replaced atomically, so an interrupted run does not leave a partial report.

Both files use UTF-8 CSV with a header. Invoices require `reference,currency,amount`; payments require `id,reference,currency,amount`. Quoted fields and embedded commas are supported; multiline fields are rejected with a line number. Identifiers must be unique in each input, amounts must be positive with at most two decimal places, and currency must be three uppercase letters. The supported precision is two decimal places; do not supply currencies requiring a different scale. Invalid input aborts before writing a report. Exit 0 means a report was produced; exit 2 means invalid arguments or data. Reconciliation exceptions are business results, not process failures.

This is a local batch tool. Reports and source files remain on your machine. No service or account is required. Maven/.NET dependencies are bundled for offline builds.
