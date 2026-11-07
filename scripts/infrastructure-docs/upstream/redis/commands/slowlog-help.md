# SLOWLOG HELP

Show helpful text about the different subcommands

## Syntax

```text
SLOWLOG HELP (null)
```


**Available since** 6.2.0

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
