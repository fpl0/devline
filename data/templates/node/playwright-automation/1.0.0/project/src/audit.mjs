import { mkdir, writeFile, rename } from "node:fs/promises";
import { resolve, join } from "node:path";
import { randomUUID } from "node:crypto";
import { reportHTML } from "./report.mjs";
export function normalizeURL(value, base) {
  const url = new URL(value, base);
  if (!["http:", "https:"].includes(url.protocol)) return null;
  url.hash = "";
  return url.href;
}
export async function audit(
  browser,
  { url, output, maxPages = 10, timeout = 10000 },
) {
  const target = normalizeURL(url);
  if (!target) throw Error("Use an HTTP or HTTPS URL.");
  if (!Number.isInteger(maxPages) || maxPages < 1 || maxPages > 50)
    throw Error("maxPages must be 1–50.");
  if (!Number.isInteger(timeout) || timeout < 100 || timeout > 60000)
    throw Error("timeout must be 100–60000 ms.");
  const origin = new URL(target).origin;
  const directory = resolve(output);
  const runID = randomUUID();
  await mkdir(directory, { recursive: true });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    serviceWorkers: "block",
  });
  context.setDefaultTimeout(timeout);
  await context.route("**/*", (route) => {
    try {
      const request = new URL(route.request().url());
      if (
        request.origin === origin ||
        ["data:", "blob:"].includes(request.protocol)
      )
        return route.continue();
    } catch {}
    return route.abort("blockedbyclient");
  });
  const queue = [target],
    seen = new Set(),
    pages = [];
  const report = {
    schema_version: 1,
    target,
    started_at: new Date().toISOString(),
    max_pages: maxPages,
    truncated: false,
    findings: 0,
    pages,
  };
  try {
    while (queue.length && pages.length < maxPages) {
      const current = queue.shift();
      if (seen.has(current)) continue;
      seen.add(current);
      const page = await context.newPage();
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      const result = {
        url: current,
        title: "",
        status: null,
        findings: [],
        desktop: null,
        mobile: null,
      };
      try {
        const response = await page.goto(current, {
          waitUntil: "load",
          timeout,
        });
        result.status = response?.status() ?? null;
        if (result.status === null || result.status >= 400)
          result.findings.push(
            `HTTP request returned ${result.status ?? "no response"}.`,
          );
        if (new URL(page.url()).origin !== origin) {
          result.findings.push("Navigation left the selected origin.");
          pages.push(result);
          continue;
        }
        const snapshot = await page.evaluate(() => {
          const findings = [];
          if (!document.title.trim())
            findings.push("The page has no document title.");
          if (!document.documentElement.lang.trim())
            findings.push("The document has no language declaration.");
          const headings = document.querySelectorAll("h1");
          if (headings.length !== 1)
            findings.push(`Expected one h1 heading; found ${headings.length}.`);
          const images = [...document.images].filter(
            (img) => !img.hasAttribute("alt"),
          );
          if (images.length)
            findings.push(`${images.length} images have no alt attribute.`);
          const fields = [
            ...document.querySelectorAll(
              "input:not([type=hidden]):not([type=submit]):not([type=button]),textarea,select",
            ),
          ].filter(
            (el) =>
              !el.labels?.length &&
              !el.getAttribute("aria-label") &&
              !el.getAttribute("aria-labelledby") &&
              !el.getAttribute("title"),
          );
          if (fields.length)
            findings.push(
              `${fields.length} form controls have no associated label.`,
            );
          const buttons = [...document.querySelectorAll("button")].filter(
            (el) =>
              !el.textContent.trim() &&
              !el.getAttribute("aria-label") &&
              !el.getAttribute("aria-labelledby") &&
              !el.getAttribute("title"),
          );
          if (buttons.length)
            findings.push(
              `${buttons.length} buttons have no apparent accessible name.`,
            );
          const ids = [...document.querySelectorAll("[id]")].map((el) => el.id);
          if (new Set(ids).size !== ids.length)
            findings.push("Duplicate element IDs were found.");
          if (document.documentElement.scrollWidth > innerWidth + 1)
            findings.push("Horizontal overflow at desktop width (1280 px).");
          return {
            title: document.title,
            findings,
            links: [...document.querySelectorAll("a[href]")].map((a) => a.href),
          };
        });
        result.title = snapshot.title;
        result.findings.push(...snapshot.findings);
        const prefix = `${runID}-page-${String(pages.length + 1).padStart(2, "0")}`;
        const desktop = `${prefix}-desktop.png`,
          mobile = `${prefix}-mobile.png`;
        await page.screenshot({
          path: join(directory, desktop),
          fullPage: true,
          animations: "disabled",
        });
        result.desktop = desktop;
        await page.setViewportSize({ width: 390, height: 844 });
        if (
          await page.evaluate(
            () => document.documentElement.scrollWidth > innerWidth + 1,
          )
        )
          result.findings.push("Horizontal overflow at mobile width (390 px).");
        await page.screenshot({
          path: join(directory, mobile),
          fullPage: true,
          animations: "disabled",
        });
        result.mobile = mobile;
        for (const href of snapshot.links) {
          try {
            const link = normalizeURL(href, current);
            if (
              link &&
              new URL(link).origin === origin &&
              !seen.has(link) &&
              !queue.includes(link) &&
              !/\.(pdf|zip|png|jpe?g|gif|svg|mp[34])$/i.test(
                new URL(link).pathname,
              )
            )
              queue.push(link);
          } catch {
            result.findings.push(`Invalid link: ${href}`);
          }
        }
      } catch (error) {
        result.findings.push(
          `Navigation or browser check failed: ${error.message}`,
        );
      } finally {
        for (const error of errors.slice(0, 10))
          result.findings.push(`Uncaught JavaScript error: ${error}`);
        await page.close();
      }
      pages.push(result);
    }
  } finally {
    await context.close();
  }
  report.truncated = queue.length > 0;
  report.findings = pages.reduce((sum, page) => sum + page.findings.length, 0);
  report.finished_at = new Date().toISOString();
  await writeFile(
    join(directory, `.${runID}-report.json`),
    JSON.stringify(report, null, 2) + "\n",
  );
  await writeFile(join(directory, `.${runID}-index.html`), reportHTML(report));
  await rename(
    join(directory, `.${runID}-report.json`),
    join(directory, "report.json"),
  );
  await rename(
    join(directory, `.${runID}-index.html`),
    join(directory, "index.html"),
  );
  return report;
}
