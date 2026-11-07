# CLUSTER DELSLOTSRANGE

Sets hash slot ranges as unbound for a node.

## Syntax

```text
CLUSTER DELSLOTSRANGE start-slot end-slot [start-slot end-slot ...]
```


**Available since** 7.0.0

**Group** cluster

**Time complexity** O(N) where N is the total number of the slots between the start slot and end slot arguments.

**Arity** -4

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Arguments

```json
[
  {
    "name": "range",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "start-slot",
        "type": "integer"
      },
      {
        "name": "end-slot",
        "type": "integer"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
