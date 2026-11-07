# Look up a symbol from your editor

Open Devline on macOS once to register local lookup links. Both integrations use the active file to find the project’s documentation versions when available.

- [Visual Studio Code](vscode/README.md) adds **Devline: Look Up Symbol** to the Command Palette and editor menu.
- [Neovim](neovim/README.md) adds `:DevlineLookup` and an optional normal and visual mode mapping.

Both integrations open a URL such as `devline://search?q=std%3A%3Afs%3A%3Aread&path=%2Fproject%2Fsrc%2Fmain.rs&stack=rust`. The required `q` is the symbol. The optional `path` is an absolute local file or folder, and `stack` supplies the editor’s language hint.
