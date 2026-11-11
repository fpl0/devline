# React web app

Run `./devkit-test.sh` to type-check, execute DOM behavior tests, and build production assets with the vendored packages. Run `./devline.sh run` for the local development server at http://127.0.0.1:7480. No npm install is required.

Includes React, React Router, TanStack Query, Zustand state, Zod validation, TypeScript, Vite, Vitest, Testing Library, and Happy DOM. The task form is a runnable example; replace it with your own application. Production output is `dist/`.

The project targets macOS arm64 and Node.js 26+. Native build-tool packages are pinned for this platform; Devline detects your Node installation and does not install or upgrade it. Every package keeps its upstream license; exact transitive versions are in package-lock.json. Refresh packages using Devline's template update workflow while online, which verifies the result offline before publication.
