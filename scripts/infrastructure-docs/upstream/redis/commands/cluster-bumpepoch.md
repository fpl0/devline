# CLUSTER BUMPEPOCH

Advances the cluster config epoch.

## Syntax

```text
CLUSTER BUMPEPOCH (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 2

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Reply schema

```json
{
  "oneOf": [
    {
      "description": "if the epoch was incremented",
      "type": "string",
      "pattern": "^BUMPED [0-9]*$"
    },
    {
      "description": "if the node already has the greatest config epoch in the cluster",
      "type": "string",
      "pattern": "^STILL [0-9]*$"
    }
  ]
}
```
