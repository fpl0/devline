import { ZodError } from "zod";
import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  load,
  parseImport,
  persist,
  priorities,
  removeProject,
  saveTask,
  seed,
  statuses,
  visibleTasks,
  STORAGE_KEY,
  MAX_BACKUP_BYTES,
  serializeBoard,
  type Board,
  type Task,
} from "./model";
const labels = { open: "To do", doing: "In progress", done: "Done" };
function errorMessage(error: unknown) {
  if (error instanceof ZodError)
    return error.issues.map((issue) => issue.message).join(" ");
  return error instanceof Error
    ? error.message
    : "The change could not be saved.";
}
export function App() {
  const [initial] = useState(() => {
    try {
      return { board: load(localStorage), error: "" };
    } catch {
      return {
        board: seed(),
        error:
          "Saved data could not be read. Export the raw backup before resetting; your saved data has not been changed.",
      };
    }
  });
  const [board, setBoard] = useState<Board>(initial.board),
    [error, setError] = useState(initial.error),
    [blocked, setBlocked] = useState(Boolean(initial.error));
  const [project, setProject] = useState(""),
    [query, setQuery] = useState(""),
    [priority, setPriority] = useState(""),
    [editing, setEditing] = useState<Task | null>(null),
    [newProject, setNewProject] = useState("");
  const [notice, setNotice] = useState(""),
    [rename, setRename] = useState("");
  const importer = useRef<HTMLInputElement>(null);
  const editor = useRef<HTMLElement>(null);
  const isEditing = editing !== null;
  useEffect(() => {
    if (!isEditing) return;
    const previous = document.activeElement as HTMLElement | null;
    const handle = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        setEditing(null);
      }
      if (event.key !== "Tab") return;
      const controls = Array.from(
        editor.current?.querySelectorAll<HTMLElement>(
          "button:not(:disabled), input:not(:disabled), textarea, select",
        ) ?? [],
      );
      const first = controls[0],
        last = controls.at(-1);
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first?.focus();
      }
    };
    document.addEventListener("keydown", handle);
    return () => {
      document.removeEventListener("keydown", handle);
      previous?.focus();
    };
  }, [isEditing]);
  useEffect(() => {
    const handle = (event: StorageEvent) => {
      if (event.key === STORAGE_KEY)
        setNotice(
          "This board changed in another tab. Reload to use the newest version.",
        );
    };
    window.addEventListener("storage", handle);
    return () => window.removeEventListener("storage", handle);
  }, []);
  function commit(next: Board) {
    if (blocked) return false;
    try {
      setBoard(persist(localStorage, board, next));
      setError("");
      setNotice("Saved in this browser.");
      return true;
    } catch (e) {
      setError(errorMessage(e));
      return false;
    }
  }
  function addProject(event: FormEvent) {
    event.preventDefault();
    const name = newProject.trim();
    if (!name) return;
    const id = crypto.randomUUID();
    if (commit({ ...board, projects: [...board.projects, { id, name }] })) {
      setProject(id);
      setRename(name);
      setNewProject("");
    }
  }
  function newTask() {
    if (!board.projects.length) {
      setError("Create a project before adding a task.");
      return;
    }
    setError("");
    setEditing({
      id: crypto.randomUUID(),
      projectId: project || board.projects[0].id,
      title: "",
      notes: "",
      status: "open",
      priority: "normal",
      due: "",
    });
  }
  function exportBoard(raw = false) {
    const text = raw
      ? localStorage.getItem(STORAGE_KEY) || ""
      : serializeBoard(board);
    const url = URL.createObjectURL(
      new Blob([text], { type: "application/json" }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = raw ? "planboard-recovery.json" : "planboard.json";
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  async function importBoard(file: File) {
    try {
      if (file.size > MAX_BACKUP_BYTES)
        throw new Error("The import is larger than 5 MiB.");
      const imported = parseImport(await file.text());
      if (
        !window.confirm(
          `Replace this board with ${imported.projects.length} projects and ${imported.tasks.length} tasks? Export a backup first if needed.`,
        )
      )
        return;
      if (blocked) {
        const recovered = { ...imported, revision: imported.revision + 1 };
        serializeBoard(recovered);
        localStorage.setItem(
          STORAGE_KEY,
          JSON.stringify(recovered),
        );
        setBoard(recovered);
        setBlocked(false);
        setError("");
      } else if (!commit(imported)) return;
      setProject("");
      setEditing(null);
      setNotice("Imported the board.");
    } catch (e) {
      setError(errorMessage(e));
    }
  }
  const selected = board.projects.find((p) => p.id === project);
  const tasks = visibleTasks(board, project, query, priority);
  const done = tasks.filter((t) => t.status === "done").length;
  return (
    <div className="app-shell">
      <aside>
        <div className="brand">
          <span className="brand-mark">P</span>
          <strong>Planboard</strong>
        </div>
        <p className="sidebar-label">YOUR PROJECTS</p>
        <button
          className={`project-link ${!project ? "selected" : ""}`}
          onClick={() => setProject("")}
        >
          All projects <span>{board.tasks.length}</span>
        </button>
        {board.projects.map((p) => (
          <button
            key={p.id}
            className={`project-link ${project === p.id ? "selected" : ""}`}
            onClick={() => {
              setProject(p.id);
              setRename(p.name);
            }}
          >
            {p.name}
            <span>
              {
                board.tasks.filter(
                  (t) => t.projectId === p.id && t.status !== "done",
                ).length
              }
            </span>
          </button>
        ))}
        <form onSubmit={addProject} className="project-form">
          <label htmlFor="project-name">New project</label>
          <input
            id="project-name"
            value={newProject}
            onChange={(e) => setNewProject(e.target.value)}
            maxLength={160}
            placeholder="A worthwhile project"
            required
            disabled={blocked}
          />
          <button disabled={blocked}>Create project</button>
        </form>
        <div className="sidebar-bottom">
          <p>
            Made for a clear head.
            <br />
            Stored only in this browser.
          </p>
          <button onClick={() => exportBoard(blocked)}>
            Export {blocked ? "raw backup" : "backup"}
          </button>
          <button onClick={() => importer.current?.click()}>
            Import backup
          </button>
          <input
            ref={importer}
            type="file"
            accept="application/json,.json"
            hidden
            aria-label="Import backup file"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) void importBoard(file);
              e.target.value = "";
            }}
          />
        </div>
      </aside>
      <main>
        <header>
          <div>
            <p className="eyebrow">A LITTLE PROGRESS, EVERY DAY</p>
            <h1>{selected?.name || "Everything in view"}</h1>
            <p className="subtitle">
              {done} of {tasks.length} tasks complete
              {tasks.length
                ? ` · ${Math.round((done / tasks.length) * 100)}%`
                : ""}
            </p>
          </div>
          <button className="primary" onClick={newTask} disabled={blocked}>
            ＋ New task
          </button>
        </header>
        {error && (
          <div role="alert" className="alert">
            {error}
          </div>
        )}
        {blocked && (
          <button
            onClick={() => {
              if (
                window.confirm(
                  "Reset unreadable saved data? Export the raw backup first.",
                )
              ) {
                try {
                  localStorage.removeItem(STORAGE_KEY);
                  setBoard(seed());
                  setBlocked(false);
                  setError("");
                } catch (e) {
                  setError(errorMessage(e));
                }
              }
            }}
          >
            Reset after backing up
          </button>
        )}
        <div className="filters">
          <label className="search">
            <span>Search tasks</span>
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search titles and notes"
            />
          </label>
          <label>
            <span>Priority</span>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value)}
            >
              <option value="">All priorities</option>
              {priorities.map((p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))}
            </select>
          </label>
        </div>
        <section className="columns" aria-label="Task board">
          {statuses.map((status) => (
            <section
              key={status}
              className="column"
              aria-labelledby={`heading-${status}`}
            >
              <div className="column-heading">
                <h2 id={`heading-${status}`}>{labels[status]}</h2>
                <span>{tasks.filter((t) => t.status === status).length}</span>
              </div>
              {tasks
                .filter((t) => t.status === status)
                .map((t) => (
                  <article key={t.id} className="task">
                    <div className="card-top">
                      <span className={`priority ${t.priority}`}>
                        {t.priority}
                      </span>
                      <small>
                        {board.projects.find((p) => p.id === t.projectId)?.name}
                      </small>
                    </div>
                    <button
                      className="task-title"
                      onClick={() => setEditing({ ...t })}
                    >
                      {t.title}
                    </button>
                    {t.notes && <p>{t.notes.slice(0, 130)}</p>}
                    {t.due && (
                      <time
                        className={
                          t.due < new Date().toLocaleDateString("en-CA") &&
                          t.status !== "done"
                            ? "overdue"
                            : ""
                        }
                        dateTime={t.due}
                      >
                        Due {t.due}
                      </time>
                    )}
                    <label className="move">
                      <span className="sr-only">Status for {t.title}</span>
                      <select
                        aria-label={`Status for ${t.title}`}
                        value={t.status}
                        disabled={blocked}
                        onChange={(e) =>
                          commit(
                            saveTask(board, {
                              ...t,
                              status: e.target.value as Task["status"],
                            }),
                          )
                        }
                      >
                        {statuses.map((s) => (
                          <option key={s} value={s}>
                            {labels[s]}
                          </option>
                        ))}
                      </select>
                    </label>
                  </article>
                ))}
              {!tasks.some((t) => t.status === status) && (
                <div className="empty-column">
                  {query || priority
                    ? "No matching tasks."
                    : "A little space for what comes next."}
                </div>
              )}
            </section>
          ))}
        </section>
        {selected && (
          <details className="project-settings">
            <summary>Project settings</summary>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                commit({
                  ...board,
                  projects: board.projects.map((p) =>
                    p.id === project ? { ...p, name: rename } : p,
                  ),
                });
              }}
            >
              <label htmlFor="rename">Project name</label>
              <input
                id="rename"
                value={rename}
                onChange={(e) => setRename(e.target.value)}
                maxLength={160}
                required
              />
              <button disabled={blocked}>Rename project</button>
            </form>
            <button
              className="danger"
              disabled={blocked}
              onClick={() => {
                if (
                  window.confirm(
                    `Delete “${selected.name}” and all its tasks?`,
                  ) &&
                  commit(removeProject(board, project))
                )
                  setProject("");
              }}
            >
              Delete project and tasks
            </button>
          </details>
        )}
        <footer role="status" aria-live="polite">
          {notice ||
            "Changes are saved locally. Export a backup to keep a separate copy."}
        </footer>
      </main>
      {editing && (
        <div className="modal-backdrop">
          <section
            ref={editor}
            role="dialog"
            aria-modal="true"
            aria-labelledby="editor-title"
            className="editor"
          >
            {error && (
              <p className="alert" role="alert">
                {error}
              </p>
            )}
            <h2 id="editor-title">
              {board.tasks.some((t) => t.id === editing.id)
                ? "Edit task"
                : "New task"}
            </h2>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                try {
                  if (commit(saveTask(board, editing))) setEditing(null);
                } catch (e) {
                  setError(errorMessage(e));
                }
              }}
            >
              <label htmlFor="title">Task title</label>
              <input
                id="title"
                autoFocus
                value={editing.title}
                maxLength={160}
                required
                onChange={(e) =>
                  setEditing({ ...editing, title: e.target.value })
                }
              />
              <label htmlFor="notes">Notes</label>
              <textarea
                id="notes"
                value={editing.notes}
                maxLength={8000}
                rows={5}
                onChange={(e) =>
                  setEditing({ ...editing, notes: e.target.value })
                }
              />
              <div className="form-grid">
                <label>
                  Project
                  <select
                    value={editing.projectId}
                    onChange={(e) =>
                      setEditing({ ...editing, projectId: e.target.value })
                    }
                  >
                    {board.projects.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  Priority
                  <select
                    value={editing.priority}
                    onChange={(e) =>
                      setEditing({
                        ...editing,
                        priority: e.target.value as Task["priority"],
                      })
                    }
                  >
                    {priorities.map((p) => (
                      <option key={p}>{p}</option>
                    ))}
                  </select>
                </label>
                <label>
                  Status
                  <select
                    value={editing.status}
                    onChange={(e) =>
                      setEditing({
                        ...editing,
                        status: e.target.value as Task["status"],
                      })
                    }
                  >
                    {statuses.map((s) => (
                      <option value={s} key={s}>
                        {labels[s]}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  Due date
                  <input
                    type="date"
                    value={editing.due}
                    onChange={(e) =>
                      setEditing({ ...editing, due: e.target.value })
                    }
                  />
                </label>
              </div>
              <div className="editor-actions">
                {board.tasks.some((t) => t.id === editing.id) && (
                  <button
                    type="button"
                    className="danger"
                    onClick={() => {
                      if (
                        window.confirm("Delete this task?") &&
                        commit({
                          ...board,
                          tasks: board.tasks.filter((t) => t.id !== editing.id),
                        })
                      )
                        setEditing(null);
                    }}
                  >
                    Delete
                  </button>
                )}
                <button type="button" onClick={() => setEditing(null)}>
                  Cancel
                </button>
                <button className="primary" disabled={blocked}>
                  Save task
                </button>
              </div>
            </form>
          </section>
        </div>
      )}
    </div>
  );
}
