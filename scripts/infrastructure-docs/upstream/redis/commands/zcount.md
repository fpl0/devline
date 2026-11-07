# ZCOUNT

Returns the count of members in a sorted set that have scores within a range.

## Syntax

```text
ZCOUNT key min max
```


**Available since** 2.0.0

**Group** sorted_set

**Time complexity** O(log(N)) with N being the number of elements in the sorted set.

**Arity** 4

**ACL categories** SORTEDSET

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "min",
    "type": "double"
  },
  {
    "name": "max",
    "type": "double"
  }
]
```


## Reply schema

```json
{
  "description": "The number of elements in the specified score range",
  "type": "integer"
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
