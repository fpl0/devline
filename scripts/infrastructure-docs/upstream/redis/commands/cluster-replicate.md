# CLUSTER REPLICATE

Configure a node as replica of a master node.

## Syntax

```text
CLUSTER REPLICATE node-id
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 3

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Arguments

```json
[
  {
    "name": "node-id",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
