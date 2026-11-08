'use strict';

const { execFile } = require('node:child_process');
const { promisify } = require('node:util');

const execFileAsync = promisify(execFile);
const stackForLanguage = new Map([
  ['python', 'python'],
  ['go', 'go'],
  ['rust', 'rust'],
  ['ruby', 'ruby'],
  ['java', 'java'],
  ['csharp', 'dotnet'],
  ['fsharp', 'dotnet'],
  ['elixir', 'elixir'],
  ['phoenix-heex', 'elixir'],
  ['javascript', 'node'],
  ['javascriptreact', 'node'],
  ['typescript', 'node'],
  ['typescriptreact', 'node'],
  ['swift', 'swift'],
]);
const symbol = /[\p{L}_][\p{L}\p{N}_]*(?:(?:\.|::|#)[\p{L}_][\p{L}\p{N}_]*)*(?:\/\d+|\(\))?|(?:\|>|==|!=|<>|&&|\|\|)(?:\/\d+)?/gu;

function lookupURL(query, path, language) {
  const term = query.trim();
  if (!term || /[\r\n]/.test(term) || Buffer.byteLength(term, 'utf8') > 512) {
    return undefined;
  }
  const stack = stackForLanguage.get(language);
  const items = [['q', term]];
  if (path?.startsWith('/')) items.push(['path', path]);
  if (stack) items.push(['stack', stack]);
  return 'devline://search?' + items.map(([key, value]) => key + '=' + encodeURIComponent(value)).join('&');
}

function termFromEditor(editor) {
  const { document, selection } = editor;
  if (!selection.isEmpty) return document.getText(selection);
  const range = document.getWordRangeAtPosition(selection.active, symbol);
  return range ? document.getText(range) : '';
}

function openURL(url) {
  return execFileAsync('/usr/bin/open', [url]);
}

function activate(context, vscode = require('vscode'), open = openURL) {
  const command = vscode.commands.registerCommand('devline.lookupSelection', async () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      await vscode.window.showInformationMessage('Open an editor and select a symbol to look it up in Devline.');
      return;
    }
    const uri = editor.document.uri;
    const folders = vscode.workspace.workspaceFolders ?? [];
    const path = uri.scheme === 'file' ? uri.fsPath
      : folders.length === 1 && folders[0].uri.scheme === 'file' ? folders[0].uri.fsPath : undefined;
    const url = lookupURL(termFromEditor(editor), path, editor.document.languageId);
    if (!url) {
      await vscode.window.showInformationMessage('Select a symbol or place the cursor inside one to look it up in Devline.');
      return;
    }
    try {
      await open(url);
    } catch {
      await vscode.window.showErrorMessage('Could not open Devline. Open Devline.app once to register its lookup links.');
    }
  });
  context.subscriptions.push(command);
}

module.exports = { activate, lookupURL };
