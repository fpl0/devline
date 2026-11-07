package dev.offline.tool;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.Callable;

import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Model.CommandSpec;
import picocli.CommandLine.Option;
import picocli.CommandLine.Parameters;
import picocli.CommandLine.Spec;
import tools.jackson.databind.DeserializationFeature;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

@Command(name = "json-tool", mixinStandardHelpOptions = true, version = "1.0.0",
        description = "Validate, inspect or pretty-print a local JSON file.")
public final class JsonTool implements Callable<Integer> {
    private static final long MAX_BYTES = 8L * 1024 * 1024;
    private final JsonMapper json = JsonMapper.builder()
            .enable(DeserializationFeature.FAIL_ON_TRAILING_TOKENS).build();

    @Parameters(index = "0", paramLabel = "FILE", description = "A JSON file up to 8 MiB.")
    private Path input;

    @Option(names = "--pretty", description = "Print formatted JSON instead of a summary.")
    private boolean pretty;

    @Spec
    private CommandSpec command;

    @Override
    public Integer call() throws IOException {
        if (!Files.isRegularFile(input) || Files.size(input) > MAX_BYTES) {
            command.commandLine().getErr().println("Choose a regular JSON file up to 8 MiB.");
            return 2;
        }

        final JsonNode document;
        try {
            document = json.readTree(input.toFile());
            if (document.isMissingNode()) {
                command.commandLine().getErr().println("The input file is empty.");
                return 2;
            }
        } catch (tools.jackson.core.JacksonException error) {
            command.commandLine().getErr().println("Invalid JSON: " + error.getOriginalMessage());
            return 2;
        }

        var out = command.commandLine().getOut();
        if (pretty) {
            out.println(json.writerWithDefaultPrettyPrinter().writeValueAsString(document));
        } else {
            var summary = json.createObjectNode();
            summary.put("type", document.getNodeType().name().toLowerCase(java.util.Locale.ROOT));
            if (document.isArray() || document.isObject()) {
                summary.put("entries", document.size());
            }
            out.println(summary);
        }
        return 0;
    }

    public static void main(String[] arguments) {
        System.exit(new CommandLine(new JsonTool()).execute(arguments));
    }
}
