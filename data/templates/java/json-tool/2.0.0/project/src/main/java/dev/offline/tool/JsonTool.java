package dev.offline.tool;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;
import java.util.concurrent.Callable;
import java.util.function.Consumer;

import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Model.CommandSpec;
import picocli.CommandLine.Option;
import picocli.CommandLine.Parameters;
import picocli.CommandLine.Spec;
import tools.jackson.core.JacksonException;
import tools.jackson.databind.DeserializationFeature;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

@Command(name = "json-tool", mixinStandardHelpOptions = true, version = "2.0.0",
        description = "Inspect, format, or select a value from a JSON document.",
        subcommands = {JsonTool.Inspect.class, JsonTool.Format.class, JsonTool.Select.class})
public final class JsonTool implements Runnable {
    private static final int MAX_BYTES = 8 * 1024 * 1024;
    private static final JsonMapper JSON = JsonMapper.builder()
            .enable(DeserializationFeature.FAIL_ON_TRAILING_TOKENS)
            .build();

    @Spec
    private CommandSpec command;

    @Override
    public void run() {
        command.commandLine().usage(command.commandLine().getOut());
    }

    abstract static class InputCommand implements Callable<Integer> {
        @Spec
        CommandSpec command;

        int withDocument(String input, Consumer<JsonNode> action) {
            try {
                JsonNode document = JSON.readTree(read(input));
                if (document.isMissingNode()) {
                    return fail("The input is empty.");
                }
                action.accept(document);
                return 0;
            } catch (JacksonException error) {
                return fail("Invalid JSON: " + error.getOriginalMessage());
            } catch (IOException | IllegalArgumentException error) {
                return fail(error.getMessage());
            }
        }

        private int fail(String message) {
            command.commandLine().getErr().println(message);
            return 2;
        }

        void print(String value) {
            command.commandLine().getOut().println(value);
        }
    }

    @Command(name = "inspect", mixinStandardHelpOptions = true,
            description = "Print the top-level type and entry count.")
    static final class Inspect extends InputCommand {
        @Parameters(defaultValue = "-", paramLabel = "FILE", description = "JSON file or - for stdin.")
        private String input;

        @Override
        public Integer call() {
            return withDocument(input, document -> {
                var summary = JSON.createObjectNode();
                summary.put("type", document.getNodeType().name().toLowerCase(Locale.ROOT));
                if (document.isArray() || document.isObject()) {
                    summary.put("entries", document.size());
                }
                print(summary.toString());
            });
        }
    }

    @Command(name = "format", mixinStandardHelpOptions = true,
            description = "Print the document as JSON.")
    static final class Format extends InputCommand {
        @Parameters(defaultValue = "-", paramLabel = "FILE", description = "JSON file or - for stdin.")
        private String input;

        @Option(names = "--compact", description = "Print on one line.")
        private boolean compact;

        @Override
        public Integer call() {
            return withDocument(input, document -> print(compact
                    ? JSON.writeValueAsString(document)
                    : JSON.writerWithDefaultPrettyPrinter().writeValueAsString(document)));
        }
    }

    @Command(name = "select", mixinStandardHelpOptions = true,
            description = "Print the JSON value at an RFC 6901 JSON Pointer.")
    static final class Select extends InputCommand {
        @Parameters(index = "0", paramLabel = "POINTER", description = "For example /items/0.")
        private String pointer;

        @Parameters(index = "1", defaultValue = "-", paramLabel = "FILE",
                description = "JSON file or - for stdin.")
        private String input;

        @Override
        public Integer call() {
            return withDocument(input, document -> {
                if (!pointer.isEmpty() && !pointer.startsWith("/")) {
                    throw new IllegalArgumentException("A JSON Pointer must be empty or start with /.");
                }
                JsonNode value = document.at(pointer);
                if (value.isMissingNode()) {
                    throw new IllegalArgumentException("No JSON value at " + pointer);
                }
                print(JSON.writeValueAsString(value));
            });
        }
    }

    private static byte[] read(String input) throws IOException {
        if ("-".equals(input)) {
            return readBounded(System.in);
        }
        Path file = Path.of(input);
        if (!Files.isRegularFile(file)) {
            throw new IOException("Choose a regular JSON file or - for stdin.");
        }
        try (InputStream stream = Files.newInputStream(file)) {
            return readBounded(stream);
        }
    }

    private static byte[] readBounded(InputStream stream) throws IOException {
        byte[] bytes = stream.readNBytes(MAX_BYTES + 1);
        if (bytes.length > MAX_BYTES) {
            throw new IOException("The input exceeds 8 MiB.");
        }
        return bytes;
    }

    public static void main(String[] arguments) {
        System.exit(new CommandLine(new JsonTool()).execute(arguments));
    }
}
