# Package shelf catalog

One file per runtime, `<stack>.json`, lists the packages Devline can keep on this Mac
so they can be added to a project without a connection.

```json
{
  "schema_version": 1,
  "stack": "python",
  "registry": "PyPI",
  "packages": [
    {"name": "requests", "summary": "HTTP client with a simple API.", "category": "HTTP"}
  ]
}
```

- `name` is what the runtime's package manager calls the package. Versions are not
  listed here: the newest release that works with the local toolchain is resolved when
  the shelf is downloaded.
- `summary` is one plain sentence, at most 90 characters, saying what it is for.
- `category` is one of: Web, HTTP, Database, Data, Serialization, Testing, CLI, Async,
  Logging, Security, Cloud, AI, UI, Tooling, Utilities.

Runtime-specific fields:

- **python**: `name` may carry extras, for example `uvicorn[standard]`.
- **go**: `name` is the module path. Add `imports` (a list of package paths) only when
  the module root is not itself an importable package.
- **rust**: optional `features` (list) and `default_features` (boolean).
- **java**: `name` is `groupId:artifactId`.
- **swift**: `name` is the repository name, with `url` (https git URL) and `products`
  (library product names to link).
