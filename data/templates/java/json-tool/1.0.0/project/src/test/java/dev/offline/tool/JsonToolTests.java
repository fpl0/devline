package dev.offline.tool;

import java.io.PrintWriter;
import java.io.StringWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import picocli.CommandLine;

import static org.junit.jupiter.api.Assertions.*;

final class JsonToolTests {
    @TempDir Path directory;

    @Test void reportsJsonObjectEntries() throws Exception {
        var input = Files.writeString(directory.resolve("tasks.json"), "{\"todo\":[1,2],\"done\":[]}");
        var output = new StringWriter();
        var command = new CommandLine(new JsonTool()).setOut(new PrintWriter(output));
        assertEquals(0, command.execute(input.toString()));
        assertEquals("{\"type\":\"object\",\"entries\":2}", output.toString().strip());
    }

    @Test void rejectsTrailingJsonAndDoesNotEmitSummary() throws Exception {
        var input = Files.writeString(directory.resolve("invalid.json"), "{} []");
        var output = new StringWriter();
        var errors = new StringWriter();
        var command = new CommandLine(new JsonTool()).setOut(new PrintWriter(output)).setErr(new PrintWriter(errors));
        assertEquals(2, command.execute(input.toString()));
        assertTrue(errors.toString().contains("Invalid JSON"));
        assertTrue(output.toString().isEmpty());
    }
}
