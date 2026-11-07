# SINTERSTORE

Stores the intersect of multiple sets in a key.

## Syntax

```text
SINTERSTORE destination key [key ...]
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(N*M) worst case where N is the cardinality of the smallest set and M is the number of sets.

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
  "description": "Number of the elements in the result set.",
  "type": "integer",
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
