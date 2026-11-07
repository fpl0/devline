# Browser sources and notices

The supplied browser is the Playwright WebKit build for Apple Silicon. WebKit includes BSD-licensed and LGPL-licensed components. The corresponding WebKit revision is `5b391e6d3b14d549908dad2c6695999291313f30`. Upstream license and patent notices are preserved under `licenses/`. The exact Playwright source pin and patch are included under `playwright-patch/`.

- WebKit source tree: https://github.com/WebKit/WebKit/tree/5b391e6d3b14d549908dad2c6695999291313f30
- Playwright sources: https://github.com/microsoft/playwright/tree/v1.65.0
- WebKit license overview: https://webkit.org/licensing-webkit/
- Exact browser download: https://cdn.playwright.dev/dbazure/download/playwright/builds/webkit/2388/webkit-mac-26-arm64.zip

To get the WebKit source, run:

```sh
git init webkit-source
git -C webkit-source fetch --depth=1 https://github.com/WebKit/WebKit.git 5b391e6d3b14d549908dad2c6695999291313f30
git -C webkit-source checkout --detach FETCH_HEAD
git -C webkit-source apply /path/to/browser-archive/playwright-patch/bootstrap.diff
```

The WebKit libraries are supplied as separate frameworks. The browser package preserves the publisher's layout and binaries.
