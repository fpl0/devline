# SLOWLOG RESET

Clears all entries from the slow log.

## Syntax

```text
SLOWLOG RESET (null)
```


**Available since** 2.2.12

**Group** server

**Time complexity** O(N) where N is the number of entries in the slowlog

**Arity** 2

**Command flags** ADMIN, LOADING, STALE

## Reply schema

```json
{
  "const": "OK"
}
```
