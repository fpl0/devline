import { execFileSync, spawnSync } from 'node:child_process';
import { strict as assert } from 'node:assert';
import { test } from 'node:test';

test('summarizes a UTF-8 file', () => {
  const result = execFileSync(process.execPath, ['src/summary.mjs', 'fixtures/notes.txt'], {
    encoding: 'utf8',
  });
  assert.deepEqual(JSON.parse(result), { lines: 2, words: 4, bytes: 23 });
});

test('reports unreadable input with a failing exit status', () => {
  const result = spawnSync(process.execPath, ['src/summary.mjs', 'missing.txt'], { encoding: 'utf8' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /missing\.txt/);
});
