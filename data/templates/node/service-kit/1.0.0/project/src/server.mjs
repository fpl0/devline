import 'dotenv/config';
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import { buildApp } from './app.mjs';

const url = process.env.DEVLINE_APP_URL ? new URL(process.env.DEVLINE_APP_URL) : null;
const host = url?.hostname.replace(/^\[|\]$/g, '') ?? process.env.HOST ?? '127.0.0.1';
const port = url ? Number(url.port || 80) : Number(process.env.PORT || 3000);
const database = process.env.DATABASE_PATH || 'data/tasks.sqlite';
mkdirSync(dirname(database), { recursive: true });
const app = await buildApp({ database, logger: true });
await app.listen({ host, port });
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, () => app.close());
}
