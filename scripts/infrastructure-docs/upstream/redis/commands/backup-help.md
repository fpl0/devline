# BACKUP HELP

Return helpful text about BACKUP command parameters.

## Syntax

```text
BACKUP HELP (null)
```


**Available since** 8.10.0

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
