# JSONL Report

Count values in a JSON Lines file or stream. For example, count HTTP status values in local event logs.

```sh
./devline.sh run
printf '%s\n' '{"http":{"status":200}}' '{"http":{"status":503}}' | ./devline.sh exec ./.devline/scripts/run.sh --field /http/status
```

The field is a JSON Pointer. Counts are sorted by frequency, with stable lexical tie order. Missing fields and objects or arrays are counted separately. Bad JSON or a record larger than 1 MiB produces a line-numbered error. The program writes a single JSON object to standard output.

The pinned offline closure includes Clap for arguments and help, Serde for serialization, and serde_json for records and JSON Pointer navigation. `./devkit-test.sh` tests counts, scalar types, malformed input, and the record size limit with networking disabled by Devline.

See `dependency-coverage.json` for the exact shipped package versions and size.
