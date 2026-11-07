# Devline Lookup for Visual Studio Code

Open Devline once on the Mac to register its local lookup links. Install the extension from this directory.

```sh
npx --yes @vscode/vsce package
code --install-extension devline-lookup-0.1.0.vsix
```

Reload the editor. Select a symbol or put the cursor inside a qualified name and choose **Devline: Look Up Symbol** from the Command Palette or editor context menu. To assign a shortcut, search for that command in Keyboard Shortcuts.

The command passes the symbol, local file path, and language hint to Devline through a local URL. Run `npm test` here to exercise command registration and URL encoding.
