# ACL HELP

Returns helpful text about the different subcommands.

## Syntax

```text
ACL HELP (null)
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "type": "array",
  "description": "A list of subcommands and their description",
  "items": {
    "type": "string"
  }
}
```
