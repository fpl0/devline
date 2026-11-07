# SLOWLOG LEN

Returns the number of entries in the slow log.

## Syntax

```text
SLOWLOG LEN (null)
```


**Available since** 2.2.12

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** ADMIN, LOADING, STALE

## Reply schema

```json
{
  "type": "integer",
  "description": "Number of entries in the slow log.",
  "minimum": 0
}
```
