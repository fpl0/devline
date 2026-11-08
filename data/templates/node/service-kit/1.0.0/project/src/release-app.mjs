import Fastify from 'fastify';
import { z } from 'zod';

const release = z.object({
  title: z.string().trim().min(1).max(120).regex(/^[^\r\n]+$/),
  notes: z.string().min(1).max(16_000),
});
const releaseId = z.string().uuid();

export function buildApp(desk, options = {}) {
  const app = Fastify({ bodyLimit: 128_000, ...options });

  app.get('/', async () => ({
    app: 'Release Desk',
    submit: 'POST /releases with {title, notes}',
    results: ['GET /releases/:id', 'GET /releases/:id/artifact', 'GET /releases/:id/audit'],
    discovery: ['GET /search?q=title', 'GET /events'],
  }));
  app.get('/health', async () => ({ status: 'ready' }));
  app.post('/releases', async (request, reply) => {
    const input = release.safeParse(request.body);
    if (!input.success) return reply.code(400).send({ error: 'Provide a title and release notes.' });
    const id = await desk.submit(input.data);
    return reply.code(202).header('location', `/releases/${id}`).send({ id, status: 'queued' });
  });

  function byId(action) {
    return async (request, reply) => {
      const id = releaseId.safeParse(request.params.id);
      if (!id.success) return reply.code(404).send({ error: 'Release not found.' });
      const value = await action(id.data);
      if (value === null) return reply.code(404).send({ error: 'Release result is unavailable.' });
      return value;
    };
  }

  app.get('/releases/:id', byId(id => desk.status(id)));
  app.get('/releases/:id/audit', byId(id => desk.audit(id)));
  app.get('/releases/:id/artifact', byId(async id => {
    const notes = await desk.artifact(id);
    return notes === null ? null : { notes };
  }));
  app.get('/search', async (request, reply) => {
    const query = z.string().trim().min(1).max(120).safeParse(request.query.q);
    if (!query.success) return reply.code(400).send({ error: 'Provide a search term of 1–120 characters.' });
    return { hits: await desk.search(query.data) };
  });
  app.get('/events', async () => ({ events: desk.events() }));
  return app;
}
