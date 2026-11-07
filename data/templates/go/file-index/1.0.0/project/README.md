# File Index

Index a directory as JSON Lines with relative paths, byte counts, and SHA-256 hashes. Use it to compare release directories, feed a script, or verify a copied set of files.

```sh
GOWORK=off GOTOOLCHAIN=local GOPROXY=off go run . --root ./sample
```

Each regular file produces one JSON object on standard output. Paths are traversed in lexical order and symlinks are skipped. Errors go to standard error and return a failing exit status. Send redirected output to a path outside the indexed directory.

`./devkit-test.sh` verifies record order, known hashes, and symlink handling offline. The starter uses Go's standard `flag`, `encoding/json`, `crypto/sha256`, `io/fs`, and `testing` packages. Its dependency coverage has no third-party modules to fetch.

See `dependency-coverage.json` for the exact shipped package versions and size.
