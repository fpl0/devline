# C# JSON CLI

Format JSON and extract a value with a JSON Pointer. The example catalog and the .NET 10 SDK are enough to run it.

```sh
./devline.sh test
./devline.sh run
dotnet run -- format examples/catalog.json
dotnet run -- get /books/0/title examples/catalog.json
printf '%s' '{"a/b":{"ready":true}}' | dotnet run -- get /a~1b/ready -
```

Use `-` to read standard input. The empty pointer selects the whole document. Use `~1` for a slash and `~0` for a tilde in a key. Results go to standard output. Invalid JSON, unreadable files, and missing paths print an error to standard error and return exit code 1. Invalid command syntax returns 2.

Run `dotnet publish -c Release -o out` to build a framework-dependent executable at `out/json-cli`. It uses the locally installed .NET 10 runtime.
