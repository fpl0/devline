package dev.offline.tool;

import java.io.PrintWriter;
import java.io.StringWriter;
import java.nio.file.Files;
import java.nio.file.Path;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import picocli.CommandLine;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class JsonToolTests {
    @TempDir
    Path temp;

    @Test
    void selectsAnEscapedPropertyAndPreservesJsonNull() throws Exception {
        Path input = Files.writeString(temp.resolve("data.json"), "{\"a/b\": null}");
        var output = new StringWriter();
        var tool = new CommandLine(new JsonTool()).setOut(new PrintWriter(output, true));

        assertEquals(0, tool.execute("select", "/a~1b", input.toString()));
        assertEquals("null", output.toString().strip());
    }

    @Test
    void rejectsTrailingJsonAndWritesTheErrorToStderr() throws Exception {
        Path input = Files.writeString(temp.resolve("invalid.json"), "{} {}");
        var output = new StringWriter();
        var errors = new StringWriter();
        var tool = new CommandLine(new JsonTool())
                .setOut(new PrintWriter(output, true))
                .setErr(new PrintWriter(errors, true));

        assertEquals(2, tool.execute("inspect", input.toString()));
        assertTrue(output.toString().isEmpty());
        assertTrue(errors.toString().contains("Invalid JSON"));
    }
}
