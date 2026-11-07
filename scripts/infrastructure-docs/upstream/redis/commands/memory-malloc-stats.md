# MEMORY MALLOC-STATS

Returns the allocator statistics.

## Syntax

```text
MEMORY MALLOC-STATS (null)
```


**Available since** 4.0.0

**Group** server

**Time complexity** Depends on how much memory is allocated, could be slow

**Arity** 2

## Reply schema

```json
{
  "type": "string",
  "description": "The memory allocator's internal statistics report."
}
```
