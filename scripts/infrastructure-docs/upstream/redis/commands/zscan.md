# ZSCAN

Iterates over members and scores of a sorted set.

## Syntax

```text
ZSCAN key cursor [MATCH pattern] [COUNT count]
```


**Available since** 2.8.0

**Group** sorted_set

**Time complexity** O(1) for every call. O(N) for a complete iteration, including enough command calls for the cursor to return back to 0. N is the number of elements inside the collection.

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "cursor",
    "type": "integer"
  },
  {
    "token": "MATCH",
    "name": "pattern",
    "type": "pattern",
    "optional": true
  },
  {
    "token": "COUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "cursor and scan response in array form",
  "type": "array",
  "minItems": 2,
  "maxItems": 2,
  "items": [
    {
      "description": "cursor",
      "type": "string"
    },
    {
      "description": "list of elements of the sorted set, where each even element is the member, and each odd value is its associated score",
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  ]
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
