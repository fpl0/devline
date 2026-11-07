# CLUSTER COUNT-FAILURE-REPORTS

Returns the number of active failure reports active for a node.

## Syntax

```text
CLUSTER COUNT-FAILURE-REPORTS node-id
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the number of failure reports

**Arity** 3

**Command flags** ADMIN, LOADING, STALE

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
  "description": "the number of active failure reports for the node",
  "type": "integer",
  "minimum": 0
}
```
