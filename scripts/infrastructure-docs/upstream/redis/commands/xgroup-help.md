# XGROUP HELP

Returns helpful text about the different subcommands.

## Syntax

```text
XGROUP HELP (null)
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** 2

**ACL categories** STREAM

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
