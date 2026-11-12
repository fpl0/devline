import { resolve } from "node:path";
import { Repository } from "./repository.mjs";
import { buildApp } from "./app.mjs";
const url = new URL(process.env.DEVLINE_APP_URL || "http://127.0.0.1:7481");
if (
  url.protocol !== "http:" ||
  !["127.0.0.1", "localhost", "[::1]"].includes(url.hostname)
)
  throw Error("Use a loopback HTTP address for this local app.");
const database = resolve(process.env.ISSUE_DATA || ".data/issues.sqlite");
const app = await buildApp(new Repository(database), { logger: true });
for (const signal of ["SIGINT", "SIGTERM"])
  process.once(signal, () => {
    app.close().catch((error) => {
      console.error(error);
      process.exitCode = 1;
    });
  });
try {
  await app.listen({
    host: url.hostname.replace(/^\[|\]$/g, ""),
    port: Number(url.port || 80),
  });
  console.log(`Issue Desk data: ${database}`);
} catch (error) {
  app.log.error(error);
  await app.close();
  process.exitCode = 1;
}
