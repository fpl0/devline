# ARDEL

Deletes elements at the specified indices in an array.

## Syntax

```text
ARDEL key index [index ...]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(N) where N is the number of indices to delete

**Arity** -3

**ACL categories** ARRAY

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "index",
    "type": "integer",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "Number of elements deleted.",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "DELETE"
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
