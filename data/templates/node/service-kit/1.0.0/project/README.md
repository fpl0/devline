# Node.js service kit

Run `./devkit-test.sh` to verify the supplied dependencies and the working task API offline. Run `node src/server.mjs` to start it at http://127.0.0.1:3000. No install step is needed. POST a JSON `{ "title": "Build offline" }` to `/tasks`, GET `/tasks` to read the persisted list, and DELETE `/tasks/1` to remove a task. SQLite data stays in `data/tasks.sqlite` across restarts.

The local dependency set includes Fastify, CORS and security headers, PostgreSQL (`pg`), MySQL (`mysql2`), Redis, RabbitMQ (`amqplib`), NATS, S3 (`@aws-sdk/client-s3`), JOSE/JWT, Zod, dotenv, Pino logging, Undici HTTP, TypeScript, and Node types. Node's built-in SQLite, test runner, filesystem, crypto, and streams cover local storage and utilities. Exact transitive versions and licenses accompany the packages.

The API and tests use SQLite immediately. The other database/message/storage clients are ready for your application to connect to project services; configure those in Devline or use `devline infra configure`. Copy the connection environment from the project workspace, then create clients with those values. Tests exercise actual SQLite persistence, API validation/deletion, JWT sign/verify, and client imports. They do not claim a live connection to every optional service.

Use `.env.example` as the local configuration starting point. Node.js 26+ must already be installed; Devline detects it and never manages it. For a newly opened existing npm project, Devline preserves `package-lock.json`, saves an isolated npm cache, and proves an offline install plus build/test. `devline project run -- npm test` uses that saved environment.

Build a new template release in Devline while connected to update this complete dependency set; old releases and project pins remain unchanged.
