'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { activate, lookupURL } = require('../extension');

test('lookup percent-encodes symbols and file paths as one local URL', () => {
  assert.equal(
    lookupURL(' pathlib.Path.read_text() & café ', '/Users/example/My App/main.py', 'python'),
    'devline://search?q=pathlib.Path.read_text()%20%26%20caf%C3%A9&path=%2FUsers%2Fexample%2FMy%20App%2Fmain.py&stack=python',
  );
  assert.equal(lookupURL('Vec::new', 'remote/path', 'rust'), 'devline://search?q=Vec%3A%3Anew&stack=rust');
  assert.equal(lookupURL('useState', undefined, 'typescriptreact'), 'devline://search?q=useState&stack=node');
  assert.equal(lookupURL('View', undefined, 'swift'), 'devline://search?q=View&stack=swift');
  assert.equal(lookupURL('line one\nline two', undefined, 'python'), undefined);
  assert.equal(lookupURL('é'.repeat(257), undefined, 'python'), undefined);
});

test('command hands the exact selected symbol and local file to the launcher', async () => {
  const selection = { isEmpty: false };
  let command;
  const launched = [];
  const context = { subscriptions: [] };
  activate(context, {
    commands: {
      registerCommand(name, callback) {
        assert.equal(name, 'devline.lookupSelection');
        command = callback;
        return { dispose() {} };
      },
    },
    window: {
      activeTextEditor: {
        selection,
        document: {
          uri: { scheme: 'file', fsPath: '/project/test.go' },
          languageId: 'go',
          getText(range) {
            assert.equal(range, selection);
            return 'http.Handler';
          },
        },
      },
    },
    workspace: {},
  }, async url => launched.push(url));
  await command();
  assert.deepEqual(launched, ['devline://search?q=http.Handler&path=%2Fproject%2Ftest.go&stack=go']);
  assert.equal(context.subscriptions.length, 1);
});
