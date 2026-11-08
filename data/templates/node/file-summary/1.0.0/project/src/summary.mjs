import { readFile } from 'node:fs/promises';

const inputs = process.argv.slice(2);

if (inputs.length !== 1) {
  console.error('Usage: node src/summary.mjs FILE');
  process.exitCode = 2;
} else {
  try {
    const bytes = await readFile(inputs[0]);
    const lines = bytes.toString('utf8').split(/\r\n|\r|\n/);
    if (lines.at(-1) === '') lines.pop();
    const words = lines.join(' ').match(/\S+/gu)?.length ?? 0;
    console.log(JSON.stringify({ lines: lines.length, words, bytes: bytes.length }));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
