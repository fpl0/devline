# SUNIONSTORE

Stores the union of multiple sets in a key.

## Syntax

```text
SUNIONSTORE destination key [key ...]
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(N) where N is the total number of elements in all given sets.

**Arity** -3

**ACL categories** SET

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "destination",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 1,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "Number of the elements in the resulting set.",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "OW",
      "UPDATE"
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
  },
  {
    "flags": [
      "RO",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 2
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
