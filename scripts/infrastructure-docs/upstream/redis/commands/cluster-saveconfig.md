# CLUSTER SAVECONFIG

Forces a node to save the cluster configuration to disk.

## Syntax

```text
CLUSTER SAVECONFIG (null)
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
