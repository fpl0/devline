import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { buildApp } from '../src/release-app.mjs';
import '../src/release-desk.mjs';

test('accepts valid releases and exposes the status URL', async () => {
  const id = '6eb5a644-2a0d-4aad-9ca7-b3ab56b39e3b';
  const app = buildApp({
    submit(input) {
      assert.deepEqual(input, { title: 'Release 1', notes: 'Everything is ready.' });
      return id;
    },
    status: () => ({ id, title: 'Release 1', status: 'queued', source: 'postgresql' }),
  });

  try {
    const accepted = await app.inject({
      method: 'POST', url: '/releases', payload: { title: ' Release 1 ', notes: 'Everything is ready.' },
    });
    assert.equal(accepted.statusCode, 202);
    assert.deepEqual(accepted.json(), { id, status: 'queued' });
    assert.equal((await app.inject(accepted.headers.location)).json().status, 'queued');

    const rejected = await app.inject({ method: 'POST', url: '/releases', payload: { title: '' } });
    assert.equal(rejected.statusCode, 400);
    assert.equal((await app.inject('/releases/invalid')).statusCode, 404);
  } finally {
    await app.close();
  }
});
