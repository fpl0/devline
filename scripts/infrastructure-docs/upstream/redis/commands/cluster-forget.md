# CLUSTER FORGET

Removes a node from the nodes table.

## Syntax

```text
CLUSTER FORGET node-id
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
