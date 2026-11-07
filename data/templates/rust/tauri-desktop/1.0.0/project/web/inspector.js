const textarea = document.querySelector('#text');
const status = document.querySelector('#status');
const counters = {
  words: document.querySelector('#words'),
  lines: document.querySelector('#lines'),
  unicodeScalars: document.querySelector('#scalars'),
  utf8Bytes: document.querySelector('#bytes'),
};
const format = new Intl.NumberFormat();
let revision = 0;
let timer;

function showCounts(counts) {
  for (const [name, output] of Object.entries(counters)) {
    output.textContent = counts ? format.format(counts[name]) : '–';
  }
}

async function inspect(request) {
  try {
    const counts = await window.__TAURI__.core.invoke('inspect_text', { text: textarea.value });
    if (request !== revision) return;
    showCounts(counts);
    status.textContent = '';
  } catch (error) {
    if (request !== revision) return;
    showCounts(null);
    status.textContent = typeof error === 'string' ? error : 'The text could not be inspected.';
  }
}

function schedule() {
  clearTimeout(timer);
  const request = ++revision;
  timer = setTimeout(() => inspect(request), 80);
}

textarea.addEventListener('input', schedule);
document.querySelector('#sample').addEventListener('click', () => {
  textarea.value = 'Coffee, café, and 茶.\nA developer at work: 👩‍💻';
  textarea.focus();
  schedule();
});
document.querySelector('#clear').addEventListener('click', () => {
  textarea.value = '';
  textarea.focus();
  schedule();
});
