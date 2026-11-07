# COMMAND HELP

Returns helpful text about the different subcommands.

## Syntax

```text
COMMAND HELP (null)
```


**Available since** 5.0.0

**Group** server

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** LOADING, STALE, SENTINEL

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
