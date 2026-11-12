# Sales Observatory

A reproducible sales warehouse built with Python and DuckDB. Import CSV files, quarantine invalid rows, detect duplicate order records, and publish JSON or a standalone HTML report. The included dataset covers books, stationery, and prints over three months.

## Run the example

```sh
./test-offline.sh
./run.sh
```

The example imports `fixtures/sales.csv` into `data/sales.duckdb` and writes `data/report.html`. Open that file in a browser. It uses local CSS and works without a server or internet connection. Repeating the example recognizes the source fingerprint and does not duplicate sales.

In a generated project, use these commands for your own data:

```sh
./devline.sh exec python3 etl.py ingest fixtures/rejected.csv --rejects data/rejects.json
./devline.sh exec python3 etl.py report --since 2026-10-01 --until 2026-10-31 --html data/october.html
./devline.sh exec python3 etl.py --database data/other.duckdb ingest fixtures/sales.csv
```

The first command intentionally returns exit code **2** because the fixture includes invalid dates, negative quantities, and a conflicting duplicate. Valid rows are committed and rejected rows identify their source row and reason. Exit code **1** means the file or command could not be processed; **0** means success without rejected rows.

## Input contract

CSV headers must contain each of these columns exactly once:

| Column | Meaning |
|---|---|
| `order_id` | Stable identifier, 1–80 characters; one record per order |
| `date` | ISO date such as `2026-11-02` |
| `category` | Nonblank category, at most 80 characters |
| `quantity` | Whole units, 1–1,000,000 |
| `unit_price_cents` | Whole EUR cents, 0–100,000,000 |

An identical order record in another file is counted as a duplicate. Reusing its ID with changed values is rejected. A repeated byte-identical file is skipped entirely. Correct a rejected source and import the changed file to retry those records. Header errors reject the entire file before the warehouse is changed.

## Design and tests

`observatory.py` owns validation, transaction boundaries, parameterized SQL, and HTML rendering. `etl.py` owns argument parsing and exit codes. The `sales` table contains accepted facts; `imports` records source fingerprints and quality counts. Monetary arithmetic remains integer-based, and HTML escapes category names.

Tests cover repeat imports, restart persistence, date filtering, conflicting duplicates, malformed headers, quoted newlines, rejected-row output, HTML escaping, and the actual command-line interface. DuckDB extensions cannot auto-download. Dependencies are pinned in `requirements.txt` and stored in the bundled wheelhouse.

For a direct source checkout, run `./setup-offline.sh` and use `.venv/bin/python etl.py` in place of the generated project's `devline.sh exec` command. Extend the reports with new SQL aggregations; change ingestion validation and its tests together when the source format changes.
