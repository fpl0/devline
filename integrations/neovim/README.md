# Devline lookup for Neovim

Open Devline once on macOS to register its local lookup links. Add this directory to your Neovim runtime path, then enable the command. Neovim 0.11 or later is required.

```lua
vim.opt.rtp:prepend("/absolute/path/to/devline/integrations/neovim")
require("devline").setup({ keymap = "<leader>dl" })
```

Run `:DevlineLookup` for the symbol under the cursor, or `:DevlineLookup std::fs::read` for an explicit symbol. The optional mapping works in normal and visual mode. Visual lookup uses the selection without touching the clipboard.

From the repository root, test the integration with `nvim --headless -u NONE '+set rtp^=integrations/neovim' -l integrations/neovim/test.lua`.
