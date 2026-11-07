# SAVE

Synchronously saves the database(s) to disk.

## Syntax

```text
SAVE (null)
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(N) where N is the total number of keys in all databases

**Arity** 1

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT, NO_MULTI

## Reply schema

```json
{
  "const": "OK"
}
```
