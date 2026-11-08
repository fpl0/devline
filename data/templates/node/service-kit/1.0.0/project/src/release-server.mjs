import pino from 'pino';
import { buildApp } from './release-app.mjs';
import { connectDesk } from './release-desk.mjs';

const logger = pino();
try {
  const url = new URL(process.env.DEVLINE_APP_URL || 'http://127.0.0.1:7481');
  const desk = await connectDesk(logger);
  const app = buildApp(desk, { loggerInstance: logger });
  try {
    await app.listen({ host: url.hostname.replace(/^\[|\]$/g, ''), port: Number(url.port || 80) });
  } catch (error) {
    await desk.close();
    throw error;
  }
  for (const signal of ['SIGINT', 'SIGTERM']) {
    process.once(signal, async () => {
      await app.close();
      await desk.close();
    });
  }
} catch (error) {
  logger.error(error.message.startsWith('Missing DEVLINE_') ? error.message : 'Could not start Release Desk. Check the local services and app port.');
  process.exit(1);
}
