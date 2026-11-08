# Playwright automation

Run `./devline.sh run`, `./devline.sh test`, or `npm test`. The suite starts a local fixture server, fills a form using accessible locators, saves a screenshot, and checks a JSON API. It runs with the included npm packages and WebKit browser.

This release requires Node.js 26 and an Apple Silicon Mac with macOS 26.5 or later. The browser archive is checked and unpacked into a temporary directory for each test run. `BROWSER.json` records the exact Playwright release, browser build, and archive checksum. The browser files, source revision, patch, and licenses ship together.

Edit `tests/automation.spec.ts` to point at your own local app. Screenshots and failure traces go to `test-results`. Use `./test-offline.sh --headed` to see the browser in a window.

Use Devline's Build New Release action while connected to refresh the npm packages and matching WebKit browser. The refresh collects the new browser's source and licenses, and Devline runs the offline suite before installing the release.

[Playwright browsers](https://playwright.dev/docs/browsers) · [API testing](https://playwright.dev/docs/api-testing) · [Bundled browser source and licenses](browser-archive/SOURCES.md)
