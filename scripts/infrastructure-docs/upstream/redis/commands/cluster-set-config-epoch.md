# CLUSTER SET-CONFIG-EPOCH

Sets the configuration epoch for a new node.

## Syntax

```text
CLUSTER SET-CONFIG-EPOCH config-epoch
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
    "name": "config-epoch",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
