# SMEMBERS

Returns all members of a set.

## Syntax

```text
SMEMBERS key
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(N) where N is the set cardinality.

**Arity** 2

**ACL categories** SET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "All elements of the set.",
  "uniqueItems": true,
  "items": {
    "type": "string"
  }
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
