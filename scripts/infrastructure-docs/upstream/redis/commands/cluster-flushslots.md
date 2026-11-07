# CLUSTER FLUSHSLOTS

Deletes all slots information from a node.

## Syntax

```text
CLUSTER FLUSHSLOTS (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 2

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Reply schema

```json
{
  "const": "OK"
}
```
