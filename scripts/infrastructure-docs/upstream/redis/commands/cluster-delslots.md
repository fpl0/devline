# CLUSTER DELSLOTS

Sets hash slots as unbound for a node.

## Syntax

```text
CLUSTER DELSLOTS slot [slot ...]
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the total number of hash slot arguments

**Arity** -3

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Arguments

```json
[
  {
    "name": "slot",
    "type": "integer",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
