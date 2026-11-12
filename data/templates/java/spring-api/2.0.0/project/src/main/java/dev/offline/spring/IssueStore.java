package dev.offline.spring;

import java.time.Instant;
import java.util.*;
import org.springframework.http.HttpStatus;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.jdbc.core.RowMapper;
import org.springframework.jdbc.support.GeneratedKeyHolder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.web.server.ResponseStatusException;

@Service
public class IssueStore {
  private final JdbcTemplate db;
  private static final RowMapper<Issue> ROW =
      (r, n) ->
          new Issue(
              r.getLong("id"),
              r.getString("title"),
              r.getString("description"),
              r.getString("priority"),
              r.getString("status"),
              r.getLong("version"),
              r.getString("created_at"),
              r.getString("updated_at"));

  public record Input(
      String title, String description, String priority, String status, Long version) {}

  public record Event(long version, String action, String happenedAt) {}

  public IssueStore(JdbcTemplate db) {
    this.db = db;
  }

  static String text(String s, int max, String name) {
    if (s == null || s.isBlank() || s.strip().length() > max)
      throw bad(name + " must contain 1–" + max + " characters");
    return s.strip();
  }

  static ResponseStatusException bad(String detail) {
    return new ResponseStatusException(HttpStatus.BAD_REQUEST, detail);
  }

  static String priority(String s) {
    if (!Set.of("low", "normal", "high", "urgent").contains(s == null ? "" : s))
      throw bad("priority must be low, normal, high or urgent");
    return s;
  }

  static String description(String s) {
    if (s == null) return "";
    if (s.length() > 4000) throw bad("description cannot exceed 4000 characters");
    return s.strip();
  }

  static void transition(String from, String to) {
    if (to == null || !Set.of("open", "in_progress", "resolved").contains(to))
      throw bad("status must be open, in_progress or resolved");
    if (from.equals("open") && to.equals("resolved"))
      throw new ResponseStatusException(
          HttpStatus.CONFLICT, "Start work before resolving an issue");
    if (from.equals("resolved") && to.equals("in_progress"))
      throw new ResponseStatusException(
          HttpStatus.CONFLICT, "Reopen a resolved issue before starting work");
  }

  public List<Issue> list(String status, String query, int limit, int offset) {
    if (status != null && !Set.of("open", "in_progress", "resolved").contains(status))
      throw bad("Invalid status filter");
    if (limit < 1 || limit > 100 || offset < 0)
      throw bad("limit must be 1–100 and offset cannot be negative");
    String q = query == null ? "" : query.strip();
    if (q.length() > 120) throw bad("Search query is too long");
    return db.query(
        "SELECT * FROM issues WHERE (? IS NULL OR status=?) AND (LOCATE(LOWER(?),LOWER(title))>0 OR"
            + " LOCATE(LOWER(?),LOWER(description))>0) ORDER BY id DESC LIMIT ? OFFSET ?",
        ROW,
        status,
        status,
        q,
        q,
        limit,
        offset);
  }

  public Issue get(long id) {
    return db.query("SELECT * FROM issues WHERE id=?", ROW, id).stream()
        .findFirst()
        .orElseThrow(() -> new ResponseStatusException(HttpStatus.NOT_FOUND, "Issue not found"));
  }

  @Transactional
  public Issue create(Input input) {
    String title = text(input.title, 160, "title"),
        description = description(input.description),
        priority = priority(input.priority);
    String now = Instant.now().toString();
    var key = new GeneratedKeyHolder();
    db.update(
        c -> {
          var s =
              c.prepareStatement(
                  "INSERT INTO"
                      + " issues(title,description,priority,status,version,created_at,updated_at)"
                      + " VALUES(?,?,?,'open',1,?,?)",
                  new String[] {"id"});
          s.setString(1, title);
          s.setString(2, description);
          s.setString(3, priority);
          s.setString(4, now);
          s.setString(5, now);
          return s;
        },
        key);
    long id = Objects.requireNonNull(key.getKey()).longValue();
    db.update(
        "INSERT INTO issue_events(issue_id,version,action,happened_at) VALUES(?,1,'created',?)",
        id,
        now);
    return get(id);
  }

  @Transactional
  public Issue update(long id, Input input) {
    var previous = get(id);
    String title = text(input.title, 160, "title"),
        description = description(input.description),
        priority = priority(input.priority);
    if (input.version == null || input.version < 1)
      throw bad("version is required and must be positive");
    if (previous.version() != input.version)
      throw new ResponseStatusException(HttpStatus.CONFLICT, "Issue changed; reload before saving");
    transition(previous.status(), input.status);
    String now = Instant.now().toString();
    int changed =
        db.update(
            "UPDATE issues SET"
                + " title=?,description=?,priority=?,status=?,version=version+1,updated_at=? WHERE"
                + " id=? AND version=?",
            title,
            description,
            priority,
            input.status,
            now,
            id,
            input.version);
    if (changed != 1)
      throw new ResponseStatusException(HttpStatus.CONFLICT, "Issue changed; reload before saving");
    db.update(
        "INSERT INTO issue_events(issue_id,version,action,happened_at) VALUES(?,?,?,?)",
        id,
        input.version + 1,
        previous.status().equals(input.status)
            ? "edited"
            : previous.status() + " → " + input.status,
        now);
    return get(id);
  }

  public List<Event> events(long id) {
    get(id);
    return db.query(
        "SELECT version,action,happened_at FROM issue_events WHERE issue_id=? ORDER BY version",
        (r, n) -> new Event(r.getLong(1), r.getString(2), r.getString(3)),
        id);
  }
}
