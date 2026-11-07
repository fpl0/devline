# __PROJECT_NAME__

A Java command-line tool that validates local JSON and prints a summary
or a formatted document. Picocli handles arguments, Jackson reads JSON,
and JUnit exercises successful and invalid input. The Maven repository
includes the pinned libraries, test tools and packaging plugins.

Run `./test-offline.sh` with JDK 26 and Maven to build and verify the
executable JAR. Then use `java -jar target/*-cli.jar fixtures/example.json`
or add `--pretty` to format the file.
