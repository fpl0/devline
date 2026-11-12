import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Repository } from "../src/repository.mjs";
import { buildApp } from "../src/app.mjs";
const input = (title) => ({
  title,
  description: "",
  priority: "normal",
  status: "open",
  version: 0,
});
test("HTTP workflow, conflicts, filters, validation and event replay", async (t) => {
  const app = await buildApp(new Repository(":memory:"));
  t.after(() => app.close());
  const request = async (method, url, payload, status = 200) => {
    const r = await app.inject({ method, url, payload });
    assert.equal(r.statusCode, status, r.body);
    return r.json();
  };
  const created = await request(
    "POST",
    "/issues",
    { title: "Fix sync", priority: "high" },
    201,
  );
  const updated = await request("PUT", "/issues/1", {
    title: "Fix sync",
    description: "Offline reproduction",
    priority: "high",
    status: "doing",
    version: created.version,
  });
  assert.equal(updated.version, 2);
  await request("PUT", "/issues/1", { title: "stale", version: 1 }, 409);
  const commented = await request("POST", "/issues/1/comments", {
    version: 2,
    body: "Confirmed",
  });
  assert.equal(commented.history.length, 3);
  assert.equal((await request("GET", "/issues?status=doing&q=sync")).total, 1);
  assert.equal((await request("GET", "/events?after=1")).items.length, 2);
  await request("POST", "/issues", { title: " " }, 422);
  await request("POST", "/issues", { title: "x", unexpected: true }, 422);
  await request("GET", "/issues?limit=0", undefined, 422);
  await request("GET", "/issues/99", undefined, 404);
  const home = await app.inject({ url: "/" });
  assert.equal(home.statusCode, 200);
  assert.match(home.body, /Capture an issue/);
});
test("SQLite persistence, invalid transitions and rollback", () => {
  const dir = mkdtempSync(join(tmpdir(), "issue-desk-"));
  const file = join(dir, "issues.sqlite");
  let repo = new Repository(file);
  try {
    repo.create(input("100% complete"));
    assert.throws(
      () => repo.update(1, { ...input("skip"), status: "done", version: 1 }),
      /transitions/,
    );
    assert.equal(repo.get(1).version, 1);
    assert.equal(repo.events(0).length, 1);
    repo.close();
    repo = new Repository(file);
    assert.equal(repo.get(1).title, "100% complete");
    assert.equal(repo.list({ q: "%", limit: 10, offset: 0 }).total, 1);
    assert.equal(repo.list({ q: "_", limit: 10, offset: 0 }).total, 0);
    assert.throws(
      () =>
        repo.transaction(() => {
          repo.db.prepare("UPDATE issues SET title=? WHERE id=1").run("bad");
          throw Error("abort");
        }),
      /abort/,
    );
    assert.equal(repo.get(1).title, "100% complete");
  } finally {
    repo.close();
    rmSync(dir, { recursive: true, force: true });
  }
});
test("event stream emits persisted issue events and closes cleanly", async () => {
  const app = await buildApp(new Repository(":memory:"));
  const address = await app.listen({ host: "127.0.0.1", port: 0 });
  const abort = new AbortController();
  try {
    await app.inject({
      method: "POST",
      url: "/issues",
      payload: { title: "Live" },
    });
    await app.inject({
      method: "PUT",
      url: "/issues/1",
      payload: { title: "Live", status: "doing", version: 1 },
    });
    const response = await fetch(address + "/events/stream?after=0", {
      signal: abort.signal,
      headers: { "Last-Event-ID": "1" },
    });
    assert.equal(response.status, 200);
    const reader = response.body.getReader();
    const { value } = await reader.read();
    const text = new TextDecoder().decode(value);
    assert.match(text, /event: issue/);
    assert.match(text, /doing/);
    assert.doesNotMatch(text, /id: 1\n/);
    abort.abort();
    await reader.cancel().catch(() => {});
  } finally {
    abort.abort();
    await app.close();
  }
});
