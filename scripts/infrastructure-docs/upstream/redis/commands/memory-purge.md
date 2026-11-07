# MEMORY PURGE

Asks the allocator to release memory.

## Syntax

```text
MEMORY PURGE (null)
```


**Available since** 4.0.0

**Group** server

**Time complexity** Depends on how much memory is allocated, could be slow

**Arity** 2

## Reply schema

```json
{
  "const": "OK"
}
```
