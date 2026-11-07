# COMMAND COUNT

Returns a count of commands.

## Syntax

```text
COMMAND COUNT (null)
```


**Available since** 2.8.13

**Group** server

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "description": "Number of total commands in this Redis server.",
  "type": "integer"
}
```
