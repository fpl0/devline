import { DatabaseSync } from "node:sqlite";
import { mkdirSync } from "node:fs";
import { dirname } from "node:path";
export class DomainError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}
const transitions = {
  open: ["open", "doing"],
  doing: ["doing", "done"],
  done: ["done", "open"],
};
export class Repository {
  constructor(file) {
    if (file !== ":memory:") mkdirSync(dirname(file), { recursive: true });
    this.db = new DatabaseSync(file);
    this.db.exec(
      "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;",
    );
    const version = this.db.prepare("PRAGMA user_version").get().user_version;
    if (version > 1) {
      this.db.close();
      throw new Error("This database was created by a newer Issue Desk.");
    }
    if (version === 0)
      this.db.exec(
        `BEGIN;
         CREATE TABLE issues (
           id INTEGER PRIMARY KEY,
           title TEXT NOT NULL,
           description TEXT NOT NULL,
           priority TEXT NOT NULL CHECK(priority IN ('low','normal','high')),
           status TEXT NOT NULL CHECK(status IN ('open','doing','done')),
           version INTEGER NOT NULL,
           created_at TEXT NOT NULL,
           updated_at TEXT NOT NULL
         );
         CREATE TABLE events (
           id INTEGER PRIMARY KEY,
           issue_id INTEGER NOT NULL REFERENCES issues(id),
           at TEXT NOT NULL,
           kind TEXT NOT NULL,
           detail TEXT NOT NULL
         );
         CREATE INDEX events_issue ON events(issue_id,id);
         CREATE INDEX issues_status ON issues(status,id);
         PRAGMA user_version=1;
         COMMIT;`,
      );
  }
  close() {
    this.db.close();
  }
  transaction(fn) {
    this.db.exec("BEGIN IMMEDIATE");
    try {
      const result = fn();
      this.db.exec("COMMIT");
      return result;
    } catch (error) {
      this.db.exec("ROLLBACK");
      throw error;
    }
  }
  get(id) {
    const row = this.db.prepare("SELECT * FROM issues WHERE id=?").get(id);
    if (!row) throw new DomainError(404, "Issue not found.");
    return {
      ...row,
      history: this.db
        .prepare(
          "SELECT id,at,kind,detail FROM events WHERE issue_id=? ORDER BY id",
        )
        .all(id),
    };
  }
  list({ status, q, limit, offset }) {
    const query = `%${q.replace(/[\\%_]/g, "\\$&")}%`;
    const where =
      "WHERE (?='' OR status=?) AND (title LIKE ? ESCAPE '\\' OR description LIKE ? ESCAPE '\\')";
    const args = [status ?? "", status ?? "", query, query];
    const total = this.db
      .prepare(`SELECT count(*) AS total FROM issues ${where}`)
      .get(...args).total;
    const items = this.db
      .prepare(
        `SELECT * FROM issues ${where} ORDER BY id DESC LIMIT ? OFFSET ?`,
      )
      .all(...args, limit, offset);
    return { items, total, limit, offset };
  }
  event(id, kind, detail) {
    this.db
      .prepare("INSERT INTO events(issue_id,at,kind,detail) VALUES(?,?,?,?)")
      .run(id, new Date().toISOString(), kind, detail);
  }
  create(input) {
    return this.transaction(() => {
      if (input.status !== "open" || input.version !== 0)
        throw new DomainError(422, "New issues must be open with no version.");
      const at = new Date().toISOString();
      const result = this.db
        .prepare(
          "INSERT INTO issues(title,description,priority,status,version,created_at,updated_at) VALUES(?,?,?,'open',1,?,?)",
        )
        .run(input.title, input.description, input.priority, at, at);
      const id = Number(result.lastInsertRowid);
      this.event(id, "created", input.title);
      return this.get(id);
    });
  }
  update(id, input) {
    return this.transaction(() => {
      const old = this.get(id);
      if (old.version !== input.version)
        throw new DomainError(409, "Version conflict; reload before editing.");
      if (!transitions[old.status].includes(input.status))
        throw new DomainError(
          409,
          "Allowed transitions: open → doing → done → open.",
        );
      this.db
        .prepare(
          "UPDATE issues SET title=?,description=?,priority=?,status=?,version=version+1,updated_at=? WHERE id=?",
        )
        .run(
          input.title,
          input.description,
          input.priority,
          input.status,
          new Date().toISOString(),
          id,
        );
      this.event(id, "updated", `${old.status} → ${input.status}`);
      return this.get(id);
    });
  }
  comment(id, { version, body }) {
    return this.transaction(() => {
      const old = this.get(id);
      if (old.version !== version)
        throw new DomainError(
          409,
          "Version conflict; reload before commenting.",
        );
      this.db
        .prepare("UPDATE issues SET version=version+1,updated_at=? WHERE id=?")
        .run(new Date().toISOString(), id);
      this.event(id, "comment", body);
      return this.get(id);
    });
  }
  events(after, limit = 100) {
    return this.db
      .prepare("SELECT * FROM events WHERE id>? ORDER BY id LIMIT ?")
      .all(after, limit);
  }
}
