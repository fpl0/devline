import { Socket } from "./phoenix.mjs";
const $ = (selector) => document.querySelector(selector);
let state = { incidents: [], transitions: {} };
const actor = $("#actor");
actor.value = localStorage.getItem("incident-actor") || "Operator";
actor.addEventListener("change", () =>
  localStorage.setItem("incident-actor", actor.value),
);
const socket = new Socket("/socket");
const channel = socket.channel("room:lobby", {});
function announce(text) {
  $("#connection").textContent = text;
}
socket.onOpen(() => announce("Connected · changes sync live"));
socket.onClose(() => announce("Disconnected · reconnecting…"));
socket.connect();
channel
  .join()
  .receive("ok", render)
  .receive("error", () => announce("Unable to join room"));
channel.on("snapshot", render);
function send(event, data, onSuccess = () => {}) {
  channel
    .push(event, { ...data, actor: actor.value.trim() })
    .receive("ok", () => {
      $("#message").textContent = "";
      onSuccess();
    })
    .receive("error", (error) => {
      $("#message").textContent = error.message;
    })
    .receive("timeout", () => {
      $("#message").textContent =
        "Connection timed out. Review the board before retrying.";
    });
}
function element(tag, text, className) {
  const item = document.createElement(tag);
  if (text !== undefined) item.textContent = text;
  if (className) item.className = className;
  return item;
}
function render(snapshot) {
  state = snapshot;
  const container = $("#incidents");
  container.replaceChildren();
  const visible = state.incidents.filter(
    (item) => $("#closed").checked || item.status !== "resolved",
  );
  if (!visible.length)
    container.append(
      element(
        "p",
        "No active incidents. Open one when a response needs coordination.",
        "empty",
      ),
    );
  for (const item of visible) {
    const article = element("article");
    article.append(
      element("span", item.severity, "badge " + item.severity),
      element("h3", item.title),
      element("p", `#${item.id} · ${item.status} · Owner: ${item.assignee}`),
    );
    const timeline = element("ol");
    for (const event of item.timeline) {
      const entry = element("li");
      entry.append(
        element("strong", `${event.actor} · ${event.status}`),
        element("p", event.message),
        element("small", new Date(event.at).toLocaleString()),
      );
      timeline.append(entry);
    }
    article.append(timeline);
    const form = element("form");
    const status = element("select");
    status.setAttribute("aria-label", "Next status");
    for (const value of [item.status, ...state.transitions[item.status]]) {
      const option = element("option", value);
      option.value = value;
      status.append(option);
    }
    const owner = element("input");
    owner.value = item.assignee;
    owner.required = true;
    owner.maxLength = 60;
    owner.setAttribute("aria-label", "Incident owner");
    const note = element("textarea");
    note.required = true;
    note.maxLength = 1000;
    note.placeholder = "What changed?";
    note.setAttribute("aria-label", "Response update");
    const button = element("button", "Post update");
    form.append(status, owner, note, button);
    form.onsubmit = (e) => {
      e.preventDefault();
      send("update", {
        id: item.id,
        revision: item.revision,
        status: status.value,
        assignee: owner.value,
        message: note.value,
      });
    };
    article.append(form);
    container.append(article);
  }
}
$("#closed").onchange = () => render(state);
$("#create").onsubmit = (event) => {
  event.preventDefault();
  send(
    "create",
    { title: $("#title").value, severity: $("#severity").value },
    () => {
      $("#title").value = "";
    },
  );
};
