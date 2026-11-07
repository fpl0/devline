# CLUSTER MYSHARDID

Returns the shard ID of a node.

## Syntax

```text
CLUSTER MYSHARDID (null)
```


**Available since** 7.2.0

**Group** cluster

**Time complexity** O(1)

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "description": "the node's shard id",
  "type": "string"
}
```
