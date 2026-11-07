local M = {}

local stacks = {
  python = "python", go = "go", rust = "rust", ruby = "ruby", java = "java",
  cs = "dotnet", fsharp = "dotnet", elixir = "elixir", eelixir = "elixir", heex = "elixir",
}

local function encode(value)
  return (value:gsub("([^A-Za-z0-9._~-])", function(byte)
    return string.format("%%%02X", string.byte(byte))
  end))
end

function M.lookup_url(query, path, filetype)
  local term = vim.trim(query)
  if term == "" or term:find("[\r\n]") or #term > 512 then
    return nil
  end
  local url = "devline://search?q=" .. encode(term)
  if path and path:sub(1, 1) == "/" then
    url = url .. "&path=" .. encode(path)
  end
  if stacks[filetype] then
    url = url .. "&stack=" .. stacks[filetype]
  end
  return url
end

local function symbol_at_cursor()
  local column = vim.api.nvim_win_get_cursor(0)[2] + 1
  local line = vim.api.nvim_get_current_line()
  for start, word, finish in line:gmatch("()([%w_.:#/|>]+)()") do
    if start <= column and column < finish then
      return word:gsub("^[%.:/]+", ""):gsub("[%.:/]+$", "")
    end
  end
  return vim.fn.expand("<cword>")
end

local function selected_text()
  local mode = vim.fn.mode()
  local start = vim.fn.getpos("v")
  local finish = vim.fn.getpos(".")
  local rows = vim.fn.getregion(start, finish, { type = mode, exclusive = vim.o.selection == "exclusive" })
  return table.concat(rows, "\n")
end

function M.lookup(query)
  if vim.fn.has("mac") ~= 1 then
    vim.notify("Devline lookup requires macOS.", vim.log.levels.WARN)
    return
  end
  local path = vim.api.nvim_buf_get_name(0)
  if path == "" or vim.bo.buftype ~= "" then
    path = vim.fn.getcwd()
  end
  local url = M.lookup_url(query or symbol_at_cursor(), path, vim.bo.filetype)
  if not url then
    vim.notify("Select a single symbol or place the cursor inside one.", vim.log.levels.INFO)
    return
  end
  vim.system({ "/usr/bin/open", url }, { text = true }, function(result)
    if result.code ~= 0 then
      vim.schedule(function()
        vim.notify("Could not open Devline. Open Devline.app once to register its links.", vim.log.levels.WARN)
      end)
    end
  end)
end

function M.setup(options)
  options = options or {}
  vim.api.nvim_create_user_command("DevlineLookup", function(command)
    M.lookup(command.args ~= "" and command.args or nil)
  end, { nargs = "?", desc = "Look up a symbol in Devline", force = true })

  if options.keymap then
    vim.keymap.set("n", options.keymap, function() M.lookup() end, { desc = "Devline symbol lookup" })
    vim.keymap.set("x", options.keymap, function()
      local query = selected_text()
      M.lookup(query)
    end, { desc = "Devline lookup selection" })
  end
end

return M
