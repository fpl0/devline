# CLUSTER KEYSLOT

Returns the hash slot for a key.

## Syntax

```text
CLUSTER KEYSLOT key
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the number of bytes in the key

**Arity** 3

**Command flags** LOADING, STALE

## Arguments

```json
[
  {
    "name": "key",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The hash slot number for the specified key",
  "type": "integer",
  "minimum": 0
}
```
