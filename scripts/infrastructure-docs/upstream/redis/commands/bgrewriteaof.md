# BGREWRITEAOF

Asynchronously rewrites the append-only file to disk.

## Syntax

```text
BGREWRITEAOF (null)
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(1)

**Arity** 1

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT

## Reply schema

```json
{
  "description": "A simple string reply indicating that the rewriting started or is about to start ASAP",
  "type": "string"
}
```
