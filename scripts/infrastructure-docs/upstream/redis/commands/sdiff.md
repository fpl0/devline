# SDIFF

Returns the difference of multiple sets.

## Syntax

```text
SDIFF key [key ...]
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(N) where N is the total number of elements in all given sets.

**Arity** -2

**ACL categories** SET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "List with the members of the resulting set.",
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
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
