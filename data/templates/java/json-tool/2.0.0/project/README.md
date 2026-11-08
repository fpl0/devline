# __PROJECT_NAME__

A Java CLI for inspecting, formatting, and querying JSON. Picocli handles commands and help. Jackson parses one JSON document at a time from a file or a pipe. Commands print results to stdout and report invalid input on stderr with exit code 2.

Run the included example or build and test the executable JAR with the bundled Maven packages. You need JDK 26 and Maven 3.10 or later.

```sh
./devline.sh run
./devline.sh test
```

Try each command using the JAR built by either command above.

```sh
java -jar target/json-tool.jar --help
java -jar target/json-tool.jar inspect fixtures/example.json
java -jar target/json-tool.jar format fixtures/example.json
java -jar target/json-tool.jar select /items/0/title fixtures/example.json
printf '%s\n' '{"ready": true}' | java -jar target/json-tool.jar format --compact -
```

Each command accepts up to 8 MiB of input. Use `-` for stdin. `select` uses JSON Pointer paths, with `~1` for a slash and `~0` for a tilde inside a property name.
