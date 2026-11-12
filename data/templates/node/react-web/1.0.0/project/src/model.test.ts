import { expect, test } from "vitest";
import {
  load,
  parseImport,
  persist,
  removeProject,
  saveTask,
  seed,
  STORAGE_KEY,
  MAX_BACKUP_BYTES,
  serializeBoard,
  visibleTasks,
} from "./model";
function storage() {
  const data = new Map<string, string>();
  return {
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => {
      data.set(k, v);
    },
  };
}
test("complete task workflow persists and supports filtering and project removal", () => {
  const db = storage();
  let board = load(db);
  const task = {
    ...board.tasks[0],
    id: "new",
    title: "Book a room",
    status: "open" as const,
  };
  board = persist(db, board, saveTask(board, task));
  board = persist(db, board, saveTask(board, { ...task, status: "done" }));
  expect(load(db).tasks.find((t) => t.id === "new")?.status).toBe("done");
  expect(visibleTasks(board, "launch", "room", "high")).toHaveLength(1);
  expect(
    removeProject(board, "launch").tasks.every((t) => t.projectId !== "launch"),
  ).toBe(true);
});
test("bad dates, orphan tasks, duplicate IDs and corrupt backups are rejected", () => {
  const b = seed();
  expect(() => parseImport("{")).toThrow();
  expect(() => saveTask(b, { ...b.tasks[0], due: "2026-02-31" })).toThrow();
  expect(() =>
    parseImport(
      JSON.stringify({
        ...b,
        tasks: [{ ...b.tasks[0], projectId: "missing" }],
      }),
    ),
  ).toThrow();
  expect(() =>
    parseImport(
      JSON.stringify({ ...b, projects: [b.projects[0], b.projects[0]] }),
    ),
  ).toThrow();
});
test("stale tabs and failed storage writes do not publish changes", () => {
  const db = storage(),
    b = seed();
  persist(db, b, b);
  expect(() => persist(db, b, b)).toThrow("another tab");
  const broken = {
    getItem: () => null,
    setItem: () => {
      throw Error("quota");
    },
  };
  expect(() => persist(broken, b, { ...b, tasks: [] })).toThrow("quota");
  expect(b.tasks).toHaveLength(4);
  db.setItem(STORAGE_KEY, "bad");
  expect(() => load(db)).toThrow();
});

test("Unicode exports fit the same UTF-8 budget enforced by imports and commits", () => {
  const db = storage(),
    original = seed();
  const tasks = Array.from({ length: 220 }, (_, index) => ({
    ...original.tasks[0],
    id: `large-${index}`,
    notes: "界".repeat(8000),
  }));
  expect(() => persist(db, original, { ...original, tasks })).toThrow("5 MiB");
  expect(db.getItem(STORAGE_KEY)).toBeNull();
  const accepted = persist(db, original, {
    ...original,
    tasks: tasks.slice(0, 200),
  });
  const exported = serializeBoard(accepted);
  expect(new TextEncoder().encode(exported).byteLength).toBeLessThanOrEqual(
    MAX_BACKUP_BYTES,
  );
  expect(parseImport(exported).tasks).toHaveLength(200);
  expect(load(db).tasks).toHaveLength(200);
});
