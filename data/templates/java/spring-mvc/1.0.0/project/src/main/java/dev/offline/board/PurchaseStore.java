package dev.offline.board;

import java.math.BigDecimal;
import java.time.Instant;
import java.util.*;
import org.springframework.http.HttpStatus;
import org.springframework.jdbc.core.*;
import org.springframework.jdbc.support.GeneratedKeyHolder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;
import org.springframework.web.server.ResponseStatusException;

@Service
public class PurchaseStore {
  public record Purchase(
      long id,
      String title,
      String reason,
      long amountCents,
      String requestedBy,
      String status,
      long version,
      String decidedBy,
      String decisionNote,
      String createdAt) {
    public String amount() {
      return BigDecimal.valueOf(amountCents, 2).toPlainString();
    }
  }

  public record Event(String actor, String action, String note, String happenedAt) {}

  private static final RowMapper<Purchase> ROW =
      (r, n) ->
          new Purchase(
              r.getLong("id"),
              r.getString("title"),
              r.getString("reason"),
              r.getLong("amount_cents"),
              r.getString("requested_by"),
              r.getString("status"),
              r.getLong("version"),
              r.getString("decided_by"),
              r.getString("decision_note"),
              r.getString("created_at"));
  private final JdbcTemplate db;

  public PurchaseStore(JdbcTemplate db) {
    this.db = db;
  }

  static String text(String s, int max, String field) {
    if (s == null || s.isBlank() || s.strip().length() > max)
      throw new ResponseStatusException(
          HttpStatus.BAD_REQUEST, field + " must contain 1–" + max + " characters");
    return s.strip();
  }

  static long cents(String amount) {
    if (amount == null || !amount.matches("[0-9]+(\\.[0-9]{1,2})?"))
      throw new ResponseStatusException(
          HttpStatus.BAD_REQUEST, "Enter a positive EUR amount with at most two decimals");
    try {
      long cents = new BigDecimal(amount).movePointRight(2).longValueExact();
      if (cents < 1 || cents > 100_000_000) throw new ArithmeticException();
      return cents;
    } catch (ArithmeticException e) {
      throw new ResponseStatusException(
          HttpStatus.BAD_REQUEST, "Amount must be between EUR 0.01 and 1,000,000.00");
    }
  }

  public List<Purchase> list(String status, String q) {
    if (status != null
        && !status.isBlank()
        && !Set.of("pending", "approved", "rejected").contains(status))
      throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "Invalid status");
    return db.query(
        "SELECT * FROM purchases WHERE (?='' OR status=?) AND (LOCATE(LOWER(?),LOWER(title))>0 OR"
            + " LOCATE(LOWER(?),LOWER(requested_by))>0) ORDER BY id DESC",
        ROW,
        status == null ? "" : status,
        status == null ? "" : status,
        q,
        q);
  }

  public Purchase get(long id) {
    return db.query("SELECT * FROM purchases WHERE id=?", ROW, id).stream()
        .findFirst()
        .orElseThrow(() -> new ResponseStatusException(HttpStatus.NOT_FOUND, "Request not found"));
  }

  @Transactional
  public Purchase create(String actor, String title, String reason, String amount) {
    String checkedTitle = text(title, 160, "Title"), checkedReason = text(reason, 2000, "Reason");
    long value = cents(amount);
    String now = Instant.now().toString();
    var key = new GeneratedKeyHolder();
    db.update(
        c -> {
          var s =
              c.prepareStatement(
                  "INSERT INTO"
                      + " purchases(title,reason,amount_cents,requested_by,status,version,created_at)"
                      + " VALUES(?,?,?,?,'pending',1,?)",
                  new String[] {"id"});
          s.setString(1, checkedTitle);
          s.setString(2, checkedReason);
          s.setLong(3, value);
          s.setString(4, actor);
          s.setString(5, now);
          return s;
        },
        key);
    long id = Objects.requireNonNull(key.getKey()).longValue();
    db.update(
        "INSERT INTO purchase_events(purchase_id,actor,action,note,happened_at)"
            + " VALUES(?,?,'submitted',?,?)",
        id,
        actor,
        checkedReason,
        now);
    return get(id);
  }

  @Transactional
  public void decide(
      long id, String actor, boolean approver, long version, String decision, String note) {
    if (!approver)
      throw new ResponseStatusException(
          HttpStatus.FORBIDDEN, "Only an approver can record a decision");
    var purchase = get(id);
    if (purchase.requestedBy.equals(actor))
      throw new ResponseStatusException(
          HttpStatus.FORBIDDEN, "You cannot approve or reject your own request");
    if (!Set.of("approved", "rejected").contains(decision))
      throw new ResponseStatusException(HttpStatus.BAD_REQUEST, "Choose approve or reject");
    String checkedNote = text(note, 500, "Decision note");
    int changed =
        db.update(
            "UPDATE purchases SET status=?,version=version+1,decided_by=?,decision_note=? WHERE"
                + " id=? AND version=? AND status='pending'",
            decision,
            actor,
            checkedNote,
            id,
            version);
    if (changed != 1)
      throw new ResponseStatusException(
          HttpStatus.CONFLICT, "Request already decided or changed; reload before trying again");
    db.update(
        "INSERT INTO purchase_events(purchase_id,actor,action,note,happened_at) VALUES(?,?,?,?,?)",
        id,
        actor,
        decision,
        checkedNote,
        Instant.now().toString());
  }

  public List<Event> events(long id) {
    return db.query(
        "SELECT actor,action,note,happened_at FROM purchase_events WHERE purchase_id=? ORDER BY id",
        (r, n) -> new Event(r.getString(1), r.getString(2), r.getString(3), r.getString(4)),
        id);
  }
}
