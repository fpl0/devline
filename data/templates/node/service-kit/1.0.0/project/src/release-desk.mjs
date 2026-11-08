import { randomUUID } from 'node:crypto';
import { setTimeout as sleep } from 'node:timers/promises';
import { Pool } from 'pg';
import { createPool } from 'mysql2/promise';
import { createClient } from 'redis';
import amqp from 'amqplib';
import { connect as connectNats } from '@nats-io/transport-node';
import { GetObjectCommand, PutObjectCommand, S3Client } from '@aws-sdk/client-s3';

const bucket = 'devline';
const queue = 'node-release-desk';
const subject = 'releases.completed';
const index = 'node-releases';

function configuration() {
  const required = [
    'POSTGRES_URL', 'MYSQL_URL', 'REDIS_URL', 'RABBITMQ_URL', 'NATS_URL',
    'SEAWEEDFS_URL', 'SEAWEEDFS_PASSWORD', 'MEILISEARCH_URL', 'MEILISEARCH_PASSWORD',
    'MAILPIT_WEB_PORT',
  ];
  return Object.fromEntries(required.map(key => {
    const value = process.env[`DEVLINE_${key}`];
    if (!value) throw new Error(`Missing DEVLINE_${key}. Create and run this starter with Devline.`);
    return [key, value];
  }));
}

async function http(url, options) {
  const response = await fetch(url, {
    ...options, signal: AbortSignal.timeout(5_000), redirect: 'error',
  });
  if (!response.ok) throw new Error(`${new URL(url).pathname} returned HTTP ${response.status}`);
  return response.json();
}

export async function connectDesk(logger) {
  const env = configuration();
  const db = new Pool({ connectionString: env.POSTGRES_URL, connectionTimeoutMillis: 5_000 });
  db.on('error', () => logger.warn('PostgreSQL connection closed'));
  const auditDb = createPool(env.MYSQL_URL);
  const cache = createClient({ url: env.REDIS_URL, socket: { connectTimeout: 5_000 } });
  cache.on('error', () => logger.warn('Redis connection closed'));
  const s3 = new S3Client({
    endpoint: env.SEAWEEDFS_URL, forcePathStyle: true, region: 'us-east-1',
    credentials: { accessKeyId: 'devline', secretAccessKey: env.SEAWEEDFS_PASSWORD },
    requestChecksumCalculation: 'WHEN_REQUIRED',
  });
  const broker = await amqp.connect(env.RABBITMQ_URL);
  broker.on('error', () => logger.warn('RabbitMQ connection closed'));
  const channel = await broker.createConfirmChannel();
  channel.on('error', () => logger.warn('RabbitMQ channel closed'));
  const natsUrl = new URL(env.NATS_URL);
  const nats = await connectNats({
    servers: natsUrl.host, user: decodeURIComponent(natsUrl.username),
    pass: decodeURIComponent(natsUrl.password),
  });

  await db.query(`
    CREATE TABLE IF NOT EXISTS release_desk (
      id UUID PRIMARY KEY,
      title TEXT NOT NULL,
      notes TEXT NOT NULL,
      status TEXT NOT NULL DEFAULT 'queued',
      published_at TIMESTAMPTZ
    )
  `);
  await auditDb.execute(`
    CREATE TABLE IF NOT EXISTS release_desk_audit (
      id CHAR(36) PRIMARY KEY,
      status VARCHAR(20) NOT NULL,
      recorded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
  `);
  await cache.connect();
  await channel.assertQueue(queue, { durable: true });
  await channel.prefetch(1);
  const recent = [];
  nats.subscribe(subject, {
    callback(error, message) {
      if (error) {
        logger.warn('NATS event subscription stopped');
        return;
      }
      try {
        recent.unshift(message.json());
        recent.length = Math.min(recent.length, 20);
      } catch {
        logger.warn('Ignored a malformed release event');
      }
    },
  });
  await nats.flush();

  function meili(path, body) {
    return http(new URL(path, env.MEILISEARCH_URL), {
      method: body === undefined ? 'GET' : 'POST',
      headers: {
        authorization: `Bearer ${env.MEILISEARCH_PASSWORD}`,
        'content-type': 'application/json',
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  }

  async function indexRelease({ id, title, notes }) {
    const { taskUid } = await meili(`/indexes/${index}/documents?primaryKey=id`, [{ id, title, notes }]);
    for (let attempt = 0; attempt < 25; attempt++) {
      const task = await meili(`/tasks/${taskUid}`);
      if (task.status === 'succeeded') return;
      if (task.status === 'failed' || task.status === 'canceled') break;
      await sleep(200);
    }
    throw new Error('Meilisearch did not index the release');
  }

  function notify({ id, title }) {
    return http(`http://127.0.0.1:${env.MAILPIT_WEB_PORT}/api/v1/send`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        From: { Email: 'release-desk@example.test' },
        To: [{ Email: 'dev@example.test' }],
        Subject: `Release ready: ${title}`,
        Text: `Release ${id} was indexed and its notes are ready to download.`,
      }),
    });
  }

  async function processRelease(id) {
    const { rows: [row] } = await db.query('SELECT * FROM release_desk WHERE id = $1', [id]);
    if (!row || row.status === 'completed') return;
    await s3.send(new PutObjectCommand({
      Bucket: bucket, Key: `releases/${id}.txt`, Body: row.notes, ContentType: 'text/plain; charset=utf-8',
    }));
    await indexRelease(row);
    await auditDb.execute(
      "INSERT INTO release_desk_audit (id, status) VALUES (?, 'completed') ON DUPLICATE KEY UPDATE id = id",
      [id],
    );
    await notify(row);
    nats.publish(subject, JSON.stringify({ id, title: row.title, status: 'completed' }));
    await nats.flush();
    await db.query("UPDATE release_desk SET status = 'completed' WHERE id = $1", [id]);
  }

  await channel.consume(queue, async message => {
    if (!message) return;
    try {
      await processRelease(message.content.toString());
      channel.ack(message);
    } catch (error) {
      logger.warn({ type: error.name }, 'A release step failed. Retrying the queued job.');
      await sleep(1_000);
      try { channel.nack(message, false, true); } catch { /* Connection closed. */ }
    }
  });

  // Recover rows if the process exits between insertion and RabbitMQ confirmation.
  let dispatching = false;
  const timer = setInterval(async () => {
    if (dispatching) return;
    dispatching = true;
    try {
      const { rows } = await db.query('SELECT id FROM release_desk WHERE published_at IS NULL LIMIT 20');
      for (const { id } of rows) {
        await new Promise((resolve, reject) => {
          channel.sendToQueue(queue, Buffer.from(id), { persistent: true }, error => {
            if (error) reject(error);
            else resolve();
          });
        });
        await db.query('UPDATE release_desk SET published_at = NOW() WHERE id = $1', [id]);
      }
    } catch {
      logger.warn('Release dispatch failed. Pending rows will be retried.');
    } finally {
      dispatching = false;
    }
  }, 500);

  return {
    async submit({ title, notes }) {
      const id = randomUUID();
      await db.query('INSERT INTO release_desk (id, title, notes) VALUES ($1, $2, $3)', [id, title, notes]);
      return id;
    },
    async status(id) {
      const cached = await cache.get(`release:${id}`);
      if (cached) return { ...JSON.parse(cached), source: 'redis' };
      const { rows: [row] } = await db.query('SELECT id, title, status FROM release_desk WHERE id = $1', [id]);
      if (row?.status === 'completed') await cache.set(`release:${id}`, JSON.stringify(row), { EX: 60 });
      return row ? { ...row, source: 'postgresql' } : null;
    },
    async audit(id) {
      const [[row]] = await auditDb.execute('SELECT id, status FROM release_desk_audit WHERE id = ?', [id]);
      return row ?? null;
    },
    async artifact(id) {
      const { rowCount } = await db.query("SELECT 1 FROM release_desk WHERE id = $1 AND status = 'completed'", [id]);
      if (!rowCount) return null;
      const result = await s3.send(new GetObjectCommand({ Bucket: bucket, Key: `releases/${id}.txt` }));
      return result.Body.transformToString();
    },
    async search(query) {
      try {
        const { hits } = await meili(`/indexes/${index}/search`, { q: query, limit: 20 });
        return hits;
      } catch (error) {
        if (error.message.endsWith('HTTP 404')) return [];
        throw error;
      }
    },
    events() { return recent; },
    async close() {
      clearInterval(timer);
      await Promise.allSettled([broker.close(), nats.close(), cache.close(), db.end(), auditDb.end()]);
      s3.destroy();
    },
  };
}
