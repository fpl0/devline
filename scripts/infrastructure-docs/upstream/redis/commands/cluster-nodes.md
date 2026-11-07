# CLUSTER NODES

Returns the cluster configuration for a node.

## Syntax

```text
CLUSTER NODES (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the total number of Cluster nodes

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "description": "the serialized cluster configuration",
  "type": "string"
}
```
