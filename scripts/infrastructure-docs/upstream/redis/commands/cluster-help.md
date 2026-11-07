# CLUSTER HELP

Returns helpful text about the different subcommands.

## Syntax

```text
CLUSTER HELP (null)
```


**Available since** 5.0.0

**Group** cluster

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
