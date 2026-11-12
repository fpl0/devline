import { createServer } from "node:http";
import { readFileSync } from "node:fs";
const pages = Object.fromEntries(
  ["workshop", "broken", "not-found"].map((name) => [
    name,
    readFileSync(new URL(`../fixtures/${name}.html`, import.meta.url)),
  ]),
);
export async function fixture() {
  const server = createServer((request, response) => {
    response.setHeader("Content-Type", "text/html; charset=utf-8");
    if (request.url === "/") response.end(pages.workshop);
    else if (request.url === "/broken") response.end(pages.broken);
    else {
      response.statusCode = 404;
      response.end(pages["not-found"]);
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  return {
    url: `http://127.0.0.1:${server.address().port}`,
    close: () =>
      new Promise((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  };
}
