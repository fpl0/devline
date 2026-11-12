import { z } from "zod";
const id = z.string().min(1).max(100);
const shortText = z.string().trim().min(1).max(160);
export const statuses = ["open", "doing", "done"] as const;
export const priorities = ["low", "normal", "high"] as const;
const date = z
  .string()
  .refine(
    (value) =>
      value === "" ||
      (/^\d{4}-\d{2}-\d{2}$/.test(value) &&
        !Number.isNaN(Date.parse(value)) &&
        new Date(value).toISOString().slice(0, 10) === value),
    "Use a valid calendar date.",
  );
export const projectSchema = z.object({ id, name: shortText }).strict();
export const taskSchema = z
  .object({
    id,
    projectId: id,
    title: shortText,
    notes: z.string().max(8000),
    status: z.enum(statuses),
    priority: z.enum(priorities),
    due: date,
  })
  .strict();
export const boardSchema = z
  .object({
    schema: z.literal(1),
    revision: z.number().int().nonnegative(),
    projects: z.array(projectSchema).max(100),
    tasks: z.array(taskSchema).max(5000),
  })
  .strict()
  .superRefine((board, ctx) => {
    const projects = new Set(board.projects.map((p) => p.id));
    const tasks = new Set(board.tasks.map((t) => t.id));
    if (
      projects.size !== board.projects.length ||
      tasks.size !== board.tasks.length
    )
      ctx.addIssue({ code: "custom", message: "IDs must be unique." });
    if (board.tasks.some((t) => !projects.has(t.projectId)))
      ctx.addIssue({
        code: "custom",
        message: "Every task must belong to an existing project.",
      });
  });
export type Task = z.infer<typeof taskSchema>;
export type Project = z.infer<typeof projectSchema>;
export type Board = z.infer<typeof boardSchema>;
export const STORAGE_KEY = "planboard.v1";
export function seed(): Board {
  return {
    schema: 1,
    revision: 0,
    projects: [
      { id: "launch", name: "Launch the workshop" },
      { id: "learning", name: "Learning notebook" },
    ],
    tasks: [
      {
        id: "one",
        projectId: "launch",
        title: "Outline the first workshop",
        notes:
          "Audience: developers who want practical offline tools.\nOutcome: leave with a working project.",
        status: "doing",
        priority: "high",
        due: "",
      },
      {
        id: "two",
        projectId: "launch",
        title: "Prepare the example repository",
        notes: "Include a tested project and a short guide.",
        status: "open",
        priority: "normal",
        due: "",
      },
      {
        id: "three",
        projectId: "launch",
        title: "Pick a date",
        notes: "Leave time for one rehearsal.",
        status: "done",
        priority: "normal",
        due: "",
      },
      {
        id: "four",
        projectId: "learning",
        title: "Write a note about concurrency",
        notes: "Explain a worker pool using a concrete example.",
        status: "open",
        priority: "low",
        due: "",
      },
    ],
  };
}
export function load(storage: Pick<Storage, "getItem">): Board {
  const raw = storage.getItem(STORAGE_KEY);
  return raw === null ? seed() : parseImport(raw);
}
export const MAX_BACKUP_BYTES = 5 * 1024 * 1024;
export function serializeBoard(board: Board): string {
  const text = JSON.stringify(board, null, 2);
  if (new TextEncoder().encode(text).byteLength > MAX_BACKUP_BYTES)
    throw new Error(
      "This board exceeds the 5 MiB backup capacity. Export and remove some tasks before adding more.",
    );
  return text;
}
export function parseImport(raw: string): Board {
  if (new TextEncoder().encode(raw).byteLength > MAX_BACKUP_BYTES)
    throw new Error("The import is larger than 5 MiB.");
  const checked = boardSchema.parse(JSON.parse(raw));
  serializeBoard(checked);
  return checked;
}
export function persist(
  storage: Pick<Storage, "getItem" | "setItem">,
  previous: Board,
  next: Board,
): Board {
  const stored = storage.getItem(STORAGE_KEY);
  if (stored !== null && parseImport(stored).revision !== previous.revision)
    throw new Error(
      "This board changed in another tab. Reload before editing.",
    );
  const checked = boardSchema.parse({
    ...next,
    revision: previous.revision + 1,
  });
  serializeBoard(checked);
  storage.setItem(STORAGE_KEY, JSON.stringify(checked));
  return checked;
}
export function saveTask(board: Board, task: Task): Board {
  const checked = taskSchema.parse(task);
  const existing = board.tasks.some((t) => t.id === task.id);
  return boardSchema.parse({
    ...board,
    tasks: existing
      ? board.tasks.map((t) => (t.id === task.id ? checked : t))
      : [...board.tasks, checked],
  });
}
export function removeProject(board: Board, id: string): Board {
  return {
    ...board,
    projects: board.projects.filter((p) => p.id !== id),
    tasks: board.tasks.filter((t) => t.projectId !== id),
  };
}
export function visibleTasks(
  board: Board,
  project: string,
  query: string,
  priority: string,
): Task[] {
  const q = query.trim().toLocaleLowerCase();
  return board.tasks.filter(
    (t) =>
      (!project || t.projectId === project) &&
      (!priority || t.priority === priority) &&
      `${t.title} ${t.notes}`.toLocaleLowerCase().includes(q),
  );
}
