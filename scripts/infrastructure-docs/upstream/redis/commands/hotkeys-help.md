# HOTKEYS HELP

Return helpful text about HOTKEYS command parameters.

## Syntax

```text
HOTKEYS HELP (null)
```


**Available since** 8.6.1

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "type": "array",
  "description": "Helpful text about subcommands.",
  "items": {
    "type": "string"
  }
}
```
