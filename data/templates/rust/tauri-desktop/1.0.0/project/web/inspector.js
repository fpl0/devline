const $ = (id) => document.getElementById(id);
const invoke = (name, args = {}) => window.__TAURI__.core.invoke(name, args);
let notes = [],
  current = null,
  dirty = false,
  busy = false;
const message = (text) => {
  $("status").textContent = text;
};
function setBusy(value) {
  busy = value;
  for (const control of document.querySelectorAll("button, input, textarea"))
    control.disabled = value;
}
function statistics() {
  const text = $("body").value;
  $("statistics").textContent =
    `${text.trim() ? text.trim().split(/\s+/u).length : 0} words · ${[...text].length} characters`;
}
function changed() {
  dirty = true;
  message("Unsaved changes");
  statistics();
}
function mayLeave() {
  return (
    !busy &&
    (!dirty || window.confirm("Discard the unsaved changes to this note?"))
  );
}
function renderList() {
  const query = $("search").value.trim().toLocaleLowerCase();
  $("notes").replaceChildren();
  for (const note of notes.filter((n) =>
    `${n.title} ${n.body} ${n.tags.join(" ")}`
      .toLocaleLowerCase()
      .includes(query),
  )) {
    const button = document.createElement("button");
    button.type = "button";
    button.disabled = busy;
    button.className = `note-row ${current?.id === note.id ? "selected" : ""}`;
    const title = document.createElement("strong");
    title.textContent = note.title;
    const preview = document.createElement("span");
    preview.textContent =
      note.body.replace(/\s+/gu, " ").slice(0, 80) || "An empty page";
    const tags = document.createElement("small");
    tags.textContent = note.tags.join(" · ");
    button.append(title, preview, tags);
    button.addEventListener("click", () => {
      if (mayLeave()) edit(note);
    });
    $("notes").append(button);
  }
  if (!$("notes").children.length) {
    const p = document.createElement("p");
    p.className = "list-empty";
    p.textContent = query ? "No matching notes." : "Your notebook is empty.";
    $("notes").append(p);
  }
  $("count").textContent =
    `${notes.length} ${notes.length === 1 ? "note" : "notes"}`;
}
function edit(note) {
  current = note;
  dirty = false;
  $("empty").hidden = true;
  $("editor").hidden = false;
  $("title").value = note?.title || "";
  $("body").value = note?.body || "";
  $("tags").value = note?.tags.join(", ") || "";
  $("delete").hidden = !note;
  $("modified").textContent = note
    ? `Updated ${new Date(note.updated_at_ms).toLocaleString()}`
    : "New note";
  message("");
  statistics();
  renderList();
  $("title").focus();
}
async function refresh() {
  notes = await invoke("list_notes");
  renderList();
}
async function save() {
  if (busy || !$("editor").reportValidity()) return;
  setBusy(true);

  try {
    const note = await invoke("save_note", {
      id: current?.id ?? null,
      version: current?.version ?? 0,
      title: $("title").value,
      body: $("body").value,
      tags: $("tags")
        .value.split(",")
        .map((s) => s.trim())
        .filter(Boolean),
    });
    await refresh();
    edit(note);
    message("Saved on this Mac.");
  } catch (error) {
    message(String(error));
  } finally {
    setBusy(false);
    $("title").focus();
  }
}
$("editor").addEventListener("submit", (e) => {
  e.preventDefault();
  save();
});
for (const id of ["title", "body", "tags"])
  $(id).addEventListener("input", changed);
$("new").addEventListener("click", () => {
  if (mayLeave()) edit(null);
});
$("search").addEventListener("input", renderList);
$("delete").addEventListener("click", async () => {
  if (
    busy ||
    !current ||
    !window.confirm(`Delete “${current.title}”? This cannot be undone.`)
  )
    return;
  setBusy(true);
  try {
    await invoke("delete_note", { id: current.id, version: current.version });
    current = null;
    dirty = false;
    $("editor").hidden = true;
    $("empty").hidden = false;
    await refresh();
    message("Note deleted.");
  } catch (e) {
    message(String(e));
  } finally {
    setBusy(false);
  }
});
$("export").addEventListener("click", async () => {
  try {
    $("json").value = await invoke("export_notebook");
    $("json").readOnly = true;
    $("transfer-title").textContent = "Export your notebook";
    $("transfer-help").textContent =
      "Copy this JSON into a .json file as a backup. Only saved notes are included.";
    $("confirm-import").hidden = true;
    $("transfer-error").textContent = "";
    $("transfer").showModal();
    $("json").select();
  } catch (e) {
    message(String(e));
  }
});
$("import").addEventListener("click", () => {
  $("json").value = "";
  $("json").readOnly = false;
  $("transfer-title").textContent = "Import a notebook";
  $("transfer-help").textContent =
    "Paste a Fieldnotes export. All notes are added as new copies; existing notes stay intact.";
  $("confirm-import").hidden = false;
  $("transfer-error").textContent = "";
  $("transfer").showModal();
  $("json").focus();
});
$("confirm-import").addEventListener("click", async () => {
  if (busy) return;
  setBusy(true);

  try {
    const count = await invoke("import_notebook", { text: $("json").value });
    await refresh();
    $("transfer").close();
    message(`Imported ${count} notes.`);
  } catch (e) {
    $("transfer-error").textContent = String(e);
  } finally {
    setBusy(false);
  }
});
document.addEventListener("keydown", (e) => {
  if (
    (e.metaKey || e.ctrlKey) &&
    e.key.toLowerCase() === "s" &&
    !$("editor").hidden
  ) {
    e.preventDefault();
    save();
  }
});
window.addEventListener("beforeunload", (e) => {
  if (dirty) {
    e.preventDefault();
    e.returnValue = "";
  }
});
refresh().catch((e) => message(String(e)));
