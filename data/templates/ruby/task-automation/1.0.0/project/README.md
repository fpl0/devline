# Snapshot Vault

A Ruby command-line backup tool built with Thor. Capture a directory as an immutable manifest, deduplicate its contents by SHA-256, verify stored bytes, and restore a selected snapshot to an explicit destination.

## Try the complete workflow

```sh
./test-offline.sh
./run.sh
```

The demo captures `fixtures/workspace`, verifies it, and restores it into a temporary directory. For a durable vault in a generated project:

```sh
./devline.sh exec ruby automation.rb capture fixtures/workspace --name before-change
./devline.sh exec ruby automation.rb list
./devline.sh exec ruby automation.rb verify before-change
./devline.sh exec ruby automation.rb restore before-change ./restored-workspace
```

Use `--vault PATH` to choose another vault; the default is `data/vault`. Snapshot names contain letters, numbers, underscores, and hyphens. Existing snapshot names cannot be replaced.

Restoration verifies all objects and preflights all destination paths before writing. Existing files are protected. To intentionally replace them, pass `--overwrite`; unrelated files in the destination are retained. The tool rejects symbolic links, symbolic-link ancestors of the destination, and unsafe manifest paths. Use a real destination path rather than an alias through a symbolic link. File permissions are restored; empty directories, ownership, ACLs, extended attributes, and symlinks are outside this snapshot format.

## Storage and failure behavior

`objects/` stores one immutable byte stream per content digest. `snapshots/NAME.json` records relative paths, sizes, modes, and digests. Multiple snapshots reuse identical objects. A writer lock serializes captures; incomplete captures never publish a manifest, although unreferenced objects may remain after failure. Capture detects a source file changing while it is read and fails instead of publishing an inconsistent record.

Each restored file is written atomically. The whole directory restoration is not one filesystem transaction; a disk failure may leave a partially restored directory. Rerun `verify` to check the vault and choose a fresh destination to retry. Snapshots are unencrypted local files.

`lib/vault.rb` contains the reusable storage operations; `automation.rb` provides Thor commands and exit handling. `Rakefile` runs Minitest. Tests cover deduplication, a byte-for-byte restore, overwrite protection, preflight ordering, corrupt objects, symbolic links, symlink ancestors, and malicious relative paths.

For a direct source bundle, use `./bundle-offline.sh exec ruby automation.rb` instead of the generated project's `devline.sh exec` prefix. Add a retention policy only after explicitly defining which manifests and unreferenced objects it may remove.
