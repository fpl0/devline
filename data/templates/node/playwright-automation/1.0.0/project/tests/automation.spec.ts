import { test, expect } from "@playwright/test";
import { mkdtemp, readFile, rm, access } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
// @ts-ignore JavaScript modules intentionally form the CLI's public boundary.
import { audit, normalizeURL } from "../src/audit.mjs";
// @ts-ignore
import { fixture } from "../src/fixture.mjs";
let local: { url: string; close: () => Promise<void> };
test.beforeAll(async () => {
  local = await fixture();
});
test.afterAll(async () => {
  await local.close();
});
test("real browser workflow and bounded full audit", async ({
  page,
  browser,
}) => {
  await page.goto(local.url);
  await page.getByLabel("Your name").fill("Filipe");
  await page.getByRole("button", { name: "Reserve a place" }).click();
  await expect(page.getByRole("status")).toHaveText("Reserved for Filipe");
  const output = await mkdtemp(join(tmpdir(), "site-audit-"));
  try {
    const result = await audit(browser, {
      url: local.url,
      output,
      maxPages: 10,
      timeout: 5000,
    });
    expect(result.pages).toHaveLength(3);
    expect(result.pages[0].findings).toEqual([]);
    const broken = result.pages.find((p: any) => p.url.endsWith("/broken"));
    expect(broken.findings.join("\n")).toContain("no document title");
    expect(broken.findings.join("\n")).toContain("form controls");
    expect(broken.findings.join("\n")).toContain("mobile width");
    expect(broken.findings.join("\n")).toContain("Deliberate example error");
    expect(
      result.pages.find((p: any) => p.url.endsWith("/missing")).status,
    ).toBe(404);
    expect(result.truncated).toBe(false);
    expect(
      JSON.parse(await readFile(join(output, "report.json"), "utf8")).findings,
    ).toBeGreaterThan(0);
    expect(await readFile(join(output, "index.html"), "utf8")).toContain(
      "Site Audit",
    );
    await access(join(output, result.pages[0].desktop));
    await access(join(output, result.pages[0].mobile));
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
test("page caps and URL normalization are explicit", async ({ browser }) => {
  expect(normalizeURL("/a#part", local.url)).toBe(local.url + "/a");
  expect(normalizeURL("mailto:hello@example.com")).toBeNull();
  const output = await mkdtemp(join(tmpdir(), "site-audit-limit-"));
  try {
    const result = await audit(browser, {
      url: local.url,
      output,
      maxPages: 1,
      timeout: 5000,
    });
    expect(result.pages).toHaveLength(1);
    expect(result.truncated).toBe(true);
    await expect(
      audit(browser, { url: local.url, output, maxPages: 0 }),
    ).rejects.toThrow("maxPages");
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

test("failed screenshot capture never attributes an old artifact to a new page", async () => {
  const output = await mkdtemp(join(tmpdir(), "site-audit-failure-"));
  const fakePage = {
    on() {},
    async goto() {
      return { status: () => 200 };
    },
    url() {
      return "http://example.test/";
    },
    async evaluate() {
      return { title: "New site", findings: [], links: [] };
    },
    async screenshot() {
      throw Error("Capture failed");
    },
    async close() {},
  };
  const fakeBrowser = {
    async newContext() {
      return {
        setDefaultTimeout() {},
        async route() {},
        async newPage() {
          return fakePage;
        },
        async close() {},
      };
    },
  };
  try {
    const result = await audit(fakeBrowser, {
      url: "http://example.test/",
      output,
    });
    expect(result.pages[0].desktop).toBeNull();
    expect(result.pages[0].mobile).toBeNull();
    const html = await readFile(join(output, "index.html"), "utf8");
    expect(html).not.toContain("<img");
    expect(html).not.toContain('href="null"');
    expect(result.pages[0].findings.join(" ")).toContain("Capture failed");
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
