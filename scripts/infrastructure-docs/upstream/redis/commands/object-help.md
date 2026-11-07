# OBJECT HELP

Returns helpful text about the different subcommands.

## Syntax

```text
OBJECT HELP (null)
```


**Available since** 6.2.0

**Group** generic

**Time complexity** O(1)

**Arity** 2

**ACL categories** KEYSPACE

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
