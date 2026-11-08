local devline = require("devline")

assert(devline.lookup_url(" Enum.map/2 ", "/tmp/My App/lib.ex", "elixir")
  == "devline://search?q=Enum.map%2F2&path=%2Ftmp%2FMy%20App%2Flib.ex&stack=elixir")
assert(devline.lookup_url("A & B + café", nil, "text")
  == "devline://search?q=A%20%26%20B%20%2B%20caf%C3%A9")
assert(devline.lookup_url("useState", nil, "typescriptreact") == "devline://search?q=useState&stack=node")
assert(devline.lookup_url("View", nil, "swift") == "devline://search?q=View&stack=swift")
assert(devline.lookup_url("a\nb", nil, "python") == nil)
assert(devline.lookup_url(string.rep("é", 257), nil, "python") == nil)

local original = vim.system
local calls = {}
vim.system = function(args)
  table.insert(calls, args)
end
vim.api.nvim_buf_set_name(0, "/tmp/My App/src/main.rs")
vim.bo.filetype = "rust"
vim.api.nvim_set_current_line("let x = std::fs::read(path)")
vim.api.nvim_win_set_cursor(0, { 1, 13 })
devline.setup({ keymap = "<leader>dl" })
vim.cmd("DevlineLookup")
vim.cmd("DevlineLookup Vec::new")
assert(calls[1][1] == "/usr/bin/open")
assert(calls[1][2] == "devline://search?q=std%3A%3Afs%3A%3Aread&path=%2Ftmp%2FMy%20App%2Fsrc%2Fmain.rs&stack=rust")
assert(calls[2][2]:find("q=Vec%%3A%%3Anew"))
vim.api.nvim_set_current_line("left pathlib.Path.read_text right")
vim.api.nvim_win_set_cursor(0, { 1, 5 })
vim.cmd("normal! v21l")
vim.api.nvim_feedkeys(vim.keycode("\\dl"), "x", false)
assert(calls[3][2]:find("q=pathlib.Path.read_text&", 1, true))
vim.api.nvim_set_current_line("let value = Café.étude()")
vim.api.nvim_win_set_cursor(0, { 1, 15 })
vim.cmd("DevlineLookup")
assert(calls[4][2]:find("q=Caf%C3%A9.%C3%A9tude&", 1, true))
vim.system = original
print("Devline Neovim lookup and URL encoding passed.")
