# File Vault

A file-integrity CLI for checking an archive before and after a move or backup. It streams SHA-256 hashing through a bounded Go worker pool, saves a deterministic manifest atomically, and reports added, changed, and missing files.

## Try the complete workflow

```sh
./devline.sh test
./devline.sh run snapshot --root examples --manifest .data/inventory.json
./devline.sh run verify --root examples --manifest .data/inventory.json
printf '\nA change\n' >> examples/sample.txt
./devline.sh run verify --root examples --manifest .data/inventory.json
```

Verification prints JSON and exits **0** for a match, **1** for differences, or **2** for an input/I/O error. Pipe the JSON into another tool or retain it as backup evidence. Run `./devline.sh run snapshot --help` for flags. With no arguments, Run snapshots the included example into `.data/example-inventory.json`.

## Behaviour and boundaries

- `--workers 1..32` bounds open files and concurrent hashing; default 4.
- Manifests are capped at 64 MiB when saved and loaded; larger scans fail without replacing an existing manifest.
- Paths are relative and sorted; contents are streamed rather than loaded into memory.
- The chosen manifest file is excluded from scans, even when it lives within the root.
- Symlinks and special files are rejected. Files changing during a read fail the snapshot, leaving the previous manifest untouched.
- An empty directory is valid; empty directories themselves, metadata, permissions and timestamps are not part of an integrity comparison.
- This is an integrity checker, not a backup program or cryptographic proof of authorship. Do not modify a directory while scanning it.

`inventory.go` owns traversal and comparison, `main.go` owns the CLI and atomic manifest I/O. Tests exercise the full snapshot/change/verify cycle, malformed input, cancellation, ordering, and symlink rejection. There are no external dependencies or network requirements.
