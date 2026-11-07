# CLUSTER MYID

Returns the ID of a node.

## Syntax

```text
CLUSTER MYID (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "description": "the node id",
  "type": "string"
}
```
