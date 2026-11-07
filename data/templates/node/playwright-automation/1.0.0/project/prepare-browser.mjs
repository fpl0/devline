import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { webkit } from 'playwright';

if (process.platform !== 'darwin' || process.arch !== 'arm64')
  throw new Error('This WebKit release targets macOS Apple Silicon.');

const playwright = JSON.parse(readFileSync('node_modules/playwright/package.json')).version;
const browser = JSON.parse(readFileSync('node_modules/playwright-core/browsers.json')).browsers
  .find(entry => entry.name === 'webkit');
if (!browser || !/^\d+$/.test(browser.revision) || !/^\d+\.\d+\.\d+$/.test(playwright))
  throw new Error('Cannot determine the Playwright WebKit release.');

const webkitGit = 'https://github.com/WebKit/WebKit.git';
const playwrightTag = `https://github.com/microsoft/playwright/tree/v${playwright}`;
const patchRoot = `https://raw.githubusercontent.com/microsoft/playwright/v${playwright}/browser_patches/webkit`;
const source = `https://cdn.playwright.dev/dbazure/download/playwright/builds/webkit/${browser.revision}/webkit-mac-26-arm64.zip`;
const executable = `webkit-${browser.revision}/pw_run.sh`;
const scratch = mkdtempSync(join(process.cwd(), '.devline-webkit-'));

async function download(url) {
  const response = await fetch(url, { signal: AbortSignal.timeout(180_000) });
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}

function sha256(data) {
  return createHash('sha256').update(data).digest('hex');
}

function write(path, bytes) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes);
}

try {
  const [archive, config, patch] = await Promise.all([
    download(source),
    download(`${patchRoot}/UPSTREAM_CONFIG.sh`),
    download(`${patchRoot}/patches/bootstrap.diff`),
  ]);
  const sourceCommit = config.toString().match(/^BASE_REVISION="([a-f0-9]{40})"$/m)?.[1];
  if (!sourceCommit) throw new Error('Missing pinned WebKit source commit.');

  const sourceTree = join(scratch, 'source');
  mkdirSync(sourceTree);
  const git = args => execFileSync('/usr/bin/git', ['-C', sourceTree, ...args], {
    encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
  });
  git(['init', '-q']);
  git(['fetch', '--quiet', '--no-tags', '--depth=1', '--filter=blob:none', webkitGit, sourceCommit]);
  const licenses = git(['ls-tree', '-r', '--name-only', '-z', 'FETCH_HEAD', '--', 'Source', 'Tools'])
    .split('\0')
    .filter(path => /\/(?:COPYING|LICENSE|PATENTS|NOTICE)[.\w-]*$/i.test(path));
  for (const required of ['Source/WebCore/LICENSE-APPLE', 'Source/WebCore/LICENSE-LGPL-2',
    'Source/ThirdParty/ANGLE/LICENSE', 'Source/ThirdParty/libwebrtc/Source/LICENSE_THIRD_PARTY']) {
    if (!licenses.includes(required)) throw new Error(`Missing upstream license ${required}.`);
  }

  const packaged = join(scratch, 'archive');
  mkdirSync(packaged);
  for (let start = 0; start < licenses.length; start += 12) {
    await Promise.all(licenses.slice(start, start + 12).map(async path => {
      const url = `https://raw.githubusercontent.com/WebKit/WebKit/${sourceCommit}/${path.split('/').map(encodeURIComponent).join('/')}`;
      write(join(packaged, 'licenses', path), await download(url));
    }));
  }
  write(join(packaged, 'playwright-patch', 'UPSTREAM_CONFIG.sh'), config);
  write(join(packaged, 'playwright-patch', 'bootstrap.diff'), patch);
  write(join(packaged, 'playwright-patch', 'LICENSE'), await download(`https://raw.githubusercontent.com/microsoft/playwright/v${playwright}/LICENSE`));
  write(join(packaged, 'SOURCES.md'), `# Browser sources and notices

The supplied browser is the Playwright WebKit build for Apple Silicon. WebKit includes BSD-licensed and LGPL-licensed components. The corresponding WebKit revision is \`${sourceCommit}\`. Upstream license and patent notices are preserved under \`licenses/\`. The exact Playwright source pin and patch are included under \`playwright-patch/\`.

- WebKit source tree: https://github.com/WebKit/WebKit/tree/${sourceCommit}
- Playwright sources: ${playwrightTag}
- WebKit license overview: https://webkit.org/licensing-webkit/
- Exact browser download: ${source}

To get the WebKit source, run:

\`\`\`sh
git init webkit-source
git -C webkit-source fetch --depth=1 ${webkitGit} ${sourceCommit}
git -C webkit-source checkout --detach FETCH_HEAD
git -C webkit-source apply /path/to/browser-archive/playwright-patch/bootstrap.diff
\`\`\`

The WebKit libraries are supplied as separate frameworks. The browser package preserves the publisher's layout and binaries.
`);

  const zip = join(scratch, 'webkit.zip');
  const installed = join(scratch, 'browser', `webkit-${browser.revision}`);
  writeFileSync(zip, archive);
  execFileSync('/usr/bin/ditto', ['-x', '-k', zip, installed]);
  const minimum = execFileSync('/usr/libexec/PlistBuddy', ['-c', 'Print :LSMinimumSystemVersion',
    join(installed, 'Playwright.app', 'Contents', 'Info.plist')], { encoding: 'utf8' }).trim();
  const instance = await webkit.launch({ executablePath: join(scratch, 'browser', executable) });
  let version;
  try {
    version = instance.version();
    const page = await instance.newPage();
    await page.setContent('<button onclick="this.textContent=\'Ready\'">Start</button>');
    await page.getByRole('button', { name: 'Start' }).click({ timeout: 5000 });
    await page.getByRole('button', { name: 'Ready' }).waitFor({ timeout: 5000 });
  } finally {
    await instance.close();
  }

  const hashes = [];
  for (let start = 0, index = 0; start < archive.length; start += 45_000_000, index++) {
    const chunk = archive.subarray(start, start + 45_000_000);
    const name = `browser-${String(index).padStart(3, '0')}.part`;
    writeFileSync(join(packaged, name), chunk);
    hashes.push(`${sha256(chunk)}  ${name}`);
  }
  writeFileSync(join(packaged, 'SHA256SUMS'), hashes.join('\n') + '\n');
  const provenance = { platform: 'darwin', arch: 'arm64', kind: 'Playwright WebKit', version,
    revision: browser.revision, minimum_macos: minimum, source, source_commit: sourceCommit,
    playwright, executable, bytes: archive.length, sha256: sha256(archive) };
  rmSync('browser-archive', { recursive: true, force: true });
  renameSync(packaged, 'browser-archive');
  writeFileSync('BROWSER.json', JSON.stringify(provenance, null, 2) + '\n');
  console.log(`Archived Playwright WebKit ${version} (build ${browser.revision}), ${archive.length} bytes`);
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
