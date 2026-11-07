# CLUSTER COUNTKEYSINSLOT

Returns the number of keys in a hash slot.

## Syntax

```text
CLUSTER COUNTKEYSINSLOT slot
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 3

**Command flags** STALE

## Arguments

```json
[
  {
    "name": "slot",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "description": "The number of keys in the specified hash slot",
  "type": "integer",
  "minimum": 0
}
```
