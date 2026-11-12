import { parseArgs } from "node:util";
import { resolve } from "node:path";
import { webkit } from "playwright";
import { audit } from "./audit.mjs";
import { fixture } from "./fixture.mjs";
let local, browser;
try {
  const { values, positionals } = parseArgs({
    options: {
      url: { type: "string" },
      output: { type: "string", default: "reports/latest" },
      "max-pages": { type: "string", default: "10" },
      "timeout-ms": { type: "string", default: "10000" },
      "fail-on-findings": { type: "boolean", default: false },
      help: { type: "boolean", short: "h" },
    },
    allowPositionals: false,
  });
  if (values.help) {
    console.log(
      "Site Audit: --url URL --output DIRECTORY --max-pages 1..50 --timeout-ms 100..60000 --fail-on-findings\nWith no --url, audits the bundled local demonstration site.",
    );
  } else {
    if (!values.url) local = await fixture();
    browser = await webkit.launch({ headless: true });
    const report = await audit(browser, {
      url: values.url || local.url,
      output: values.output,
      maxPages: Number(values["max-pages"]),
      timeout: Number(values["timeout-ms"]),
    });
    console.log(
      `Audited ${report.pages.length} pages; ${report.findings} findings${report.truncated ? " (page limit reached)" : ""}.\nOpen ${resolve(values.output, "index.html")}`,
    );
    if (values["fail-on-findings"] && report.findings > 0) process.exitCode = 2;
  }
} catch (error) {
  console.error(`site-audit: ${error.message}`);
  process.exitCode = 1;
} finally {
  await browser?.close();
  await local?.close();
}
