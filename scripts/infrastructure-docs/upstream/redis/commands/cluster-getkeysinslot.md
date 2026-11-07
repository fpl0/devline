# CLUSTER GETKEYSINSLOT

Returns the key names in a hash slot.

## Syntax

```text
CLUSTER GETKEYSINSLOT slot count
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the number of requested keys

**Arity** 4

**Command flags** STALE

## Arguments

```json
[
  {
    "name": "slot",
    "type": "integer"
  },
  {
    "name": "count",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "description": "an array with up to count elements",
  "type": "array",
  "items": {
    "description": "key name",
    "type": "string"
  }
}
```
