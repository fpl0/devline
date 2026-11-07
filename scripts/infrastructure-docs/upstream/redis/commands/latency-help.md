# LATENCY HELP

Returns helpful text about the different subcommands.

## Syntax

```text
LATENCY HELP (null)
```


**Available since** 2.8.13

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
