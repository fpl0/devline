const $ = (id) => document.getElementById(id);
let selected = null,
  offset = 0,
  total = 0;
async function api(path, options = {}) {
  const response = await fetch(path, {
    ...options,
    headers: { "Content-Type": "application/json" },
  });
  const value = await response.json();
  if (!response.ok) throw Error(value.error?.message || "Request failed");
  return value;
}
function message(value, kind = "info") {
  $("message").className = kind;
  $("message").textContent = value;
}
let refreshVersion = 0;
async function refresh() {
  const requestVersion = ++refreshVersion;
  try {
    const params = new URLSearchParams({
      q: $("query").value,
      offset: String(offset),
      limit: "20",
    });
    if ($("filter").value) params.set("status", $("filter").value);
    const result = await api("/issues?" + params);
    if (requestVersion !== refreshVersion) return;
    total = result.total;
    $("list").replaceChildren();
    for (const issue of result.items) {
      const article = document.createElement("article");
      article.className = "issue";
      const badges = document.createElement("div");
      badges.className = "badges";
      badges.textContent = `#${issue.id} · ${issue.status} · ${issue.priority}`;
      const title = document.createElement("button");
      title.textContent = issue.title;
      title.onclick = () => openIssue(issue.id);
      const description = document.createElement("p");
      description.textContent = issue.description.slice(0, 150);
      article.append(badges, title, description);
      $("list").append(article);
    }
    if (!result.items.length)
      $("list").textContent = "No issues here yet. Capture one to get started.";
    $("page").textContent =
      `${total ? offset + 1 : 0}–${Math.min(offset + 20, total)} of ${total}`;
    $("previous").disabled = offset === 0;
    $("next").disabled = offset + 20 >= total;
  } catch (e) {
    message(e.message, "error");
  }
}
async function openIssue(id) {
  try {
    selected = await api("/issues/" + id);
    $("detail").hidden = false;
    $("detail-title").textContent =
      `Issue #${selected.id} · version ${selected.version}`;
    for (const field of ["title", "description", "priority", "status"])
      $("edit-" + field).value = selected[field];
    $("history").replaceChildren();
    for (const event of selected.history) {
      const li = document.createElement("li");
      li.textContent = `${new Date(event.at).toLocaleString()} · ${event.kind}: ${event.detail}`;
      $("history").append(li);
    }
  } catch (e) {
    message(e.message, "error");
  }
}
$("create").onsubmit = async (e) => {
  e.preventDefault();
  try {
    const data = Object.fromEntries(new FormData(e.target));
    const issue = await api("/issues", {
      method: "POST",
      body: JSON.stringify(data),
    });
    e.target.reset();
    offset = 0;
    await refresh();
    await openIssue(issue.id);
    message("Issue created.");
  } catch (e) {
    message(e.message, "error");
  }
};
$("edit").onsubmit = async (e) => {
  e.preventDefault();
  try {
    const data = { version: selected.version };
    for (const field of ["title", "description", "priority", "status"])
      data[field] = $("edit-" + field).value;
    await api("/issues/" + selected.id, {
      method: "PUT",
      body: JSON.stringify(data),
    });
    await openIssue(selected.id);
    await refresh();
    message("Saved.");
  } catch (e) {
    message(e.message, "error");
  }
};
$("comment").onsubmit = async (e) => {
  e.preventDefault();
  try {
    await api("/issues/" + selected.id + "/comments", {
      method: "POST",
      body: JSON.stringify({
        version: selected.version,
        body: $("comment-body").value,
      }),
    });
    $("comment-body").value = "";
    await openIssue(selected.id);
    message("Comment added.");
  } catch (e) {
    message(e.message, "error");
  }
};
$("reload").onclick = () => selected && openIssue(selected.id);
$("search").onclick = () => {
  offset = 0;
  refresh();
};
$("filter").onchange = () => {
  offset = 0;
  refresh();
};
$("query").onkeydown = (e) => {
  if (e.key === "Enter") {
    offset = 0;
    refresh();
  }
};
$("previous").onclick = () => {
  offset = Math.max(0, offset - 20);
  refresh();
};
$("next").onclick = () => {
  offset += 20;
  refresh();
};
const stream = new EventSource("/events/stream");
stream.onopen = () => {
  $("connection").textContent = "● Live updates connected";
};
stream.onerror = () => {
  $("connection").textContent = "Reconnecting…";
};
let refreshTimer;
stream.addEventListener("issue", () => {
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(refresh, 100);
});
refresh();
