import Fastify from "fastify";
import helmet from "@fastify/helmet";
import { readFileSync } from "node:fs";
import { z, ZodError } from "zod";
import { DomainError } from "./repository.mjs";
const input = z
  .object({
    title: z.string().trim().min(1).max(160),
    description: z.string().trim().max(8000).default(""),
    priority: z.enum(["low", "normal", "high"]).default("normal"),
    status: z.enum(["open", "doing", "done"]).default("open"),
    version: z.number().int().nonnegative().default(0),
  })
  .strict();
const comment = z
  .object({
    version: z.number().int().positive(),
    body: z.string().trim().min(1).max(2000),
  })
  .strict();
const filters = z
  .object({
    status: z.enum(["open", "doing", "done"]).optional(),
    q: z.string().max(160).default(""),
    limit: z.coerce.number().int().min(1).max(100).default(50),
    offset: z.coerce
      .number()
      .int()
      .min(0)
      .max(Number.MAX_SAFE_INTEGER)
      .default(0),
  })
  .strict();
const cursor = z
  .object({
    after: z.coerce
      .number()
      .int()
      .min(0)
      .max(Number.MAX_SAFE_INTEGER)
      .default(0),
  })
  .strict();
const identifier = z.coerce
  .number()
  .int()
  .positive()
  .max(Number.MAX_SAFE_INTEGER);
const dashboard = readFileSync(
  new URL("../web/index.html", import.meta.url),
  "utf8",
);
export async function buildApp(repository, { logger = false } = {}) {
  const app = Fastify({ logger, bodyLimit: 32768, requestTimeout: 10000 });
  const streams = new Set();
  await app.register(helmet, {
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        scriptSrc: ["'self'"],
        styleSrc: ["'self'"],
        connectSrc: ["'self'"],
        objectSrc: ["'none'"],
        upgradeInsecureRequests: null,
      },
    },
  });
  app.setErrorHandler((error, request, reply) => {
    if (error instanceof ZodError)
      return reply.code(422).send({
        error: {
          status: 422,
          message: error.issues
            .map((i) => `${i.path.join(".") || "input"}: ${i.message}`)
            .join("; "),
        },
      });
    if (error instanceof DomainError)
      return reply
        .code(error.status)
        .send({ error: { status: error.status, message: error.message } });
    if (error.statusCode >= 400 && error.statusCode < 500)
      return reply
        .code(error.statusCode)
        .send({ error: { status: error.statusCode, message: error.message } });
    request.log.error(error);
    return reply.code(500).send({
      error: {
        status: 500,
        message:
          "Unable to complete the request. The transaction was rolled back.",
      },
    });
  });
  app.get("/", async (_, reply) => reply.type("text/html").send(dashboard));
  app.get("/assets/app.js", async (_, reply) =>
    reply
      .type("application/javascript")
      .send(readFileSync(new URL("../web/app.js", import.meta.url), "utf8")),
  );
  app.get("/assets/styles.css", async (_, reply) =>
    reply
      .type("text/css")
      .send(
        readFileSync(new URL("../web/styles.css", import.meta.url), "utf8"),
      ),
  );
  app.get("/health", async () => ({ status: "ok" }));
  app.get("/issues", async (request) =>
    repository.list(filters.parse(request.query)),
  );
  app.get("/issues/:id", async (request) =>
    repository.get(identifier.parse(request.params.id)),
  );
  app.post("/issues", async (request, reply) => {
    const issue = repository.create(input.parse(request.body));
    return reply
      .code(201)
      .header("Location", `/issues/${issue.id}`)
      .send(issue);
  });
  app.put("/issues/:id", async (request) =>
    repository.update(
      identifier.parse(request.params.id),
      input.parse(request.body),
    ),
  );
  app.post("/issues/:id/comments", async (request) =>
    repository.comment(
      identifier.parse(request.params.id),
      comment.parse(request.body),
    ),
  );
  app.get("/events", async (request) => ({
    items: repository.events(cursor.parse(request.query).after),
  }));
  app.get("/events/stream", async (request, reply) => {
    let after = cursor.parse(request.query).after;
    const lastEvent = request.headers["last-event-id"];
    if (lastEvent !== undefined)
      after = Math.max(after, cursor.parse({ after: lastEvent }).after);
    if (streams.size >= 100)
      return reply.code(503).send({
        error: { status: 503, message: "Too many open event streams." },
      });
    reply.hijack();
    const response = reply.raw;
    response.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
      "X-Content-Type-Options": "nosniff",
    });
    let timer;
    const close = () => {
      clearInterval(timer);
      streams.delete(response);
      if (!response.writableEnded) response.end();
    };
    streams.add(response);
    response.on("close", close);
    const pump = () => {
      try {
        const events = repository.events(after);
        for (const event of events) {
          if (
            !response.write(
              `id: ${event.id}\nevent: issue\ndata: ${JSON.stringify(event)}\n\n`,
            )
          ) {
            close();
            return;
          }
          after = event.id;
        }
        if (!events.length && !response.write(": heartbeat\n\n")) close();
      } catch (error) {
        request.log.error(error);
        close();
      }
    };
    timer = setInterval(pump, 1000);
    pump();
  });
  app.addHook("preClose", async () => {
    for (const response of streams) response.end();
  });
  app.addHook("onClose", async () => repository.close());
  return app;
}
