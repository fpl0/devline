package dev.offline.spring;

import java.net.URI;
import java.util.*;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

@RestController
class IssueController {
  private final IssueStore store;

  IssueController(IssueStore store) {
    this.store = store;
  }

  @GetMapping("/")
  Map<String, String> index() {
    return Map.of("name", "Issue Desk", "issues", "/api/issues", "health", "/health");
  }

  @GetMapping("/health")
  Map<String, String> health() {
    store.list(null, "", 1, 0);
    return Map.of("status", "ready", "database", "h2");
  }

  @GetMapping("/api/issues")
  List<Issue> list(
      @RequestParam(required = false) String status,
      @RequestParam(defaultValue = "") String q,
      @RequestParam(defaultValue = "50") int limit,
      @RequestParam(defaultValue = "0") int offset) {
    return store.list(status, q, limit, offset);
  }

  @GetMapping("/api/issues/{id}")
  Issue get(@PathVariable long id) {
    return store.get(id);
  }

  @PostMapping("/api/issues")
  ResponseEntity<Issue> create(@RequestBody IssueStore.Input input) {
    var issue = store.create(input);
    return ResponseEntity.created(URI.create("/api/issues/" + issue.id())).body(issue);
  }

  @PutMapping("/api/issues/{id}")
  Issue update(@PathVariable long id, @RequestBody IssueStore.Input input) {
    return store.update(id, input);
  }

  @GetMapping("/api/issues/{id}/events")
  List<IssueStore.Event> events(@PathVariable long id) {
    return store.events(id);
  }
}
