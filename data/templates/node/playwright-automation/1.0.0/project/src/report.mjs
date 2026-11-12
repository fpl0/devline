import { readFileSync } from "node:fs";
const styles = readFileSync(new URL("./report.css", import.meta.url), "utf8");
const escapeHTML = (value) =>
  String(value).replace(
    /[&<>"']/g,
    (char) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        char
      ],
  );
function screenshot(page, kind) {
  const file = page[kind];
  if (!file) return "";
  return `<a href="${escapeHTML(file)}"><img alt="${kind} screenshot of ${escapeHTML(page.title)}" src="${escapeHTML(file)}"></a>`;
}
function pageHTML(page) {
  const findings = page.findings.length
    ? page.findings
        .map((finding) => `<li class="bad">${escapeHTML(finding)}</li>`)
        .join("\n")
    : '<li class="good">No findings from these checks.</li>';
  return `
    <article>
      <small>HTTP ${page.status ?? "unavailable"}</small>
      <h2>${escapeHTML(page.title || "Untitled page")}</h2>
      <code>${escapeHTML(page.url)}</code>
      <ul>${findings}</ul>
      <div class="screens">${screenshot(page, "desktop")}${screenshot(page, "mobile")}</div>
    </article>`;
}
export function reportHTML(report) {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width,initial-scale=1">
    <title>Site Audit</title>
    <style>${styles}</style>
  </head>
  <body>
    <h1>Site Audit</h1>
    <p>${report.pages.length} pages · ${report.findings} findings · ${escapeHTML(report.started_at)}</p>
    <p>Target: <code>${escapeHTML(report.target)}</code></p>
    <p>Heuristic content, browser-error and layout checks. This is not a complete accessibility or security audit.</p>
    ${report.pages.map(pageHTML).join("\n")}
    <footer>JSON results: <a href="report.json">report.json</a> · Screenshots contain page content; review before sharing.</footer>
  </body>
</html>`;
}
