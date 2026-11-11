[manifest | packages] = System.argv()
additions = Enum.map(packages, fn item ->
  [name, version] = String.split(item, "=", parts: 2)
  unless Regex.match?(~r/^[a-z][a-z0-9_]*$/, name), do: raise("Invalid Hex package name")
  {String.to_atom(name), "== " <> version}
end)
source = File.read!(manifest)
ast = Code.string_to_quoted!(source, columns: true, token_metadata: true)
# Prefer the conventional Mix deps/0 list. Literal positions let us insert
# without formatting away comments, quote choices, or unrelated configuration.
encoder = fn literal, metadata -> {:ok, {:__block__, metadata, [literal]}} end
literal_ast = Code.string_to_quoted!(source, columns: true, token_metadata: true, literal_encoder: encoder)
{_, lists} = Macro.prewalk(literal_ast, [], fn
  {:defp, _, [{:deps, _, arguments}, [{{:__block__, _, [:do]}, {:__block__, metadata, [items]}}]]} = node, lists
      when arguments in [nil, []] and is_list(items) ->
    {node, [{metadata, items} | lists]}
  node, lists -> {node, lists}
end)
{_, called} = Macro.prewalk(ast, false, fn
  {:def, _, [{:project, _, arguments}, [do: body]]} = node, called when arguments in [nil, []] ->
    {_, found} = Macro.prewalk(body, false, fn
      {:deps, _, args} = expression, _ when args in [nil, []] -> {expression, true}
      expression, found -> {expression, found}
    end)
    {node, called or found}
  node, called -> {node, called}
end)
if called and length(lists) == 1 do
  [{metadata, items}] = lists
  line = Keyword.fetch!(metadata, :line)
  column = Keyword.fetch!(metadata, :column)
  lines = String.split(source, "\n")
  prefix = Enum.take(lines, line - 1) |> Enum.map_join("", &(&1 <> "\n"))
  leading = Enum.at(lines, line - 1) |> String.to_charlist() |> Enum.take(column - 1) |> List.to_string()
  offset = byte_size(prefix) + byte_size(leading)
  if binary_part(source, offset, 1) == "[" do
    newline = if String.contains?(source, "\r\n"), do: "\r\n", else: "\n"
    tuples = Enum.map_join(additions, "," <> newline <> "      ", fn {name, version} -> "{:" <> Atom.to_string(name) <> ", " <> inspect(version) <> "}" end)
    inserted = newline <> "      " <> tuples <> (if items == [], do: "", else: ",") <> newline <> "    "
    updated = binary_part(source, 0, offset + 1) <> inserted <> binary_part(source, offset + 1, byte_size(source) - offset - 1)
    Code.string_to_quoted!(updated)
    File.write!(manifest, updated)
    System.halt(0)
  end
end
{_, positions} = Macro.prewalk(ast, [], fn
  {:defmodule, metadata, [_, [do: body]]} = node, positions ->
    expressions = case body do
      {:__block__, _, items} -> items
      item -> [item]
    end
    mix_project = Enum.any?(expressions, &match?({:use, _, [{:__aliases__, _, [:Mix, :Project]} | _]}, &1))
    project = Enum.count(expressions, &match?({:def, _, [{:project, _, arguments}, _]} when arguments in [nil, []], &1))
    if mix_project and project >= 1 do
      {node, [Keyword.fetch!(metadata, :end) | positions]}
    else
      {node, positions}
    end
  node, positions -> {node, positions}
end)
[position] = case positions do
  [position] -> [position]
  _ -> raise("Choose a Mix project with one explicit Mix.Project module before adding selected clients.")
end
lines = String.split(source, "\n")
line = Keyword.fetch!(position, :line)
column = Keyword.fetch!(position, :column)
prefix = Enum.take(lines, line - 1) |> Enum.map_join("", &(&1 <> "\n"))
indent = Enum.at(lines, line - 1) |> String.to_charlist() |> Enum.take(column - 1) |> List.to_string()
offset = byte_size(prefix) + byte_size(indent)
newline = if String.contains?(source, "\r\n"), do: "\r\n", else: "\n"
# Preserve the complete original function and all comments. Elixir's supported
# override mechanism extends its returned configuration without rewriting it.
insert = [
  "  # Devline: explicitly selected service clients; existing project configuration is preserved.",
  "  defoverridable project: 0",
  "  def project do",
  "    additions = " <> inspect(additions, limit: :infinity),
  "    Keyword.update(super(), :deps, additions, &(&1 ++ additions))",
  "  end",
  ""
] |> Enum.map_join(newline, &(indent <> &1))
updated = binary_part(source, 0, offset) <> insert <> binary_part(source, offset, byte_size(source) - offset)
Code.string_to_quoted!(updated)
File.write!(manifest, updated)
