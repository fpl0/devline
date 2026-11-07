# FUNCTION HELP

Returns helpful text about the different subcommands.

## Syntax

```text
FUNCTION HELP (null)
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(1)

**Arity** 2

**ACL categories** SCRIPTING

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
