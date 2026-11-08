# __PROJECT_NAME__

A small data automation starter with DuckDB, HTTPX, and pytest. The included fixtures aggregate integer cents by category with SQL. CPython 3.15 on Apple Silicon macOS is required.

Run the default CSV example with `./devline.sh run`. To pass a different file, use `./devline.sh exec python3 etl.py --input PATH`.

```sh
./setup-offline.sh
.venv/bin/python etl.py --input fixtures/sales.csv --output summary.json
.venv/bin/python etl.py --input fixtures/sales.json
./test-offline.sh
```

The `--url` option accepts a JSON endpoint when it is reachable. The test uses an HTTPX fixture transport. DuckDB reads the local CSV and JSON files with extension auto-install disabled. The locked wheelhouse installs with no index access and retains upstream notices.
