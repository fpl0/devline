package dev.offline.spring;

import java.net.URI;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentSkipListMap;
import java.util.concurrent.atomic.AtomicLong;

import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.server.ResponseStatusException;

@RestController
class TaskController {
    private final ConcurrentSkipListMap<Long, Task> tasks = new ConcurrentSkipListMap<>();
    private final AtomicLong nextId = new AtomicLong();

    record Task(long id, String title, boolean completed) {}
    record CreateTask(String title) {}
    record UpdateTask(String title, Boolean completed) {}

    @GetMapping("/")
    Map<String, String> index() {
        return Map.of("name", "Spring Boot Task API", "tasks", "/api/tasks", "health", "/health");
    }

    @GetMapping("/health")
    Map<String, String> health() {
        return Map.of("status", "up");
    }

    @GetMapping("/api/tasks")
    List<Task> list() {
        return List.copyOf(tasks.values());
    }

    @GetMapping("/api/tasks/{id}")
    Task get(@PathVariable long id) {
        return existing(tasks.get(id));
    }

    @PostMapping("/api/tasks")
    ResponseEntity<Task> create(@RequestBody CreateTask request) {
        String title = title(request.title());
        long id = nextId.incrementAndGet();
        Task task = new Task(id, title, false);
        tasks.put(id, task);
        return ResponseEntity.created(URI.create("/api/tasks/" + id)).body(task);
    }

    @PutMapping("/api/tasks/{id}")
    Task replace(@PathVariable long id, @RequestBody UpdateTask request) {
        String title = title(request.title());
        if (request.completed() == null) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "completed is required");
        }
        return existing(tasks.computeIfPresent(id,
                (key, previous) -> new Task(key, title, request.completed())));
    }

    @DeleteMapping("/api/tasks/{id}")
    ResponseEntity<Void> delete(@PathVariable long id) {
        existing(tasks.remove(id));
        return ResponseEntity.noContent().build();
    }

    private static String title(String raw) {
        String title = raw == null ? "" : raw.strip();
        if (title.isEmpty() || title.codePointCount(0, title.length()) > 120) {
            throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "title must be 1–120 characters");
        }
        return title;
    }

    private static Task existing(Task task) {
        if (task == null) {
            throw new ResponseStatusException(HttpStatus.NOT_FOUND, "Task not found");
        }
        return task;
    }
}
