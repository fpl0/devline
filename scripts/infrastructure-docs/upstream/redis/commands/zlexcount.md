# ZLEXCOUNT

Returns the number of members in a sorted set within a lexicographical range.

## Syntax

```text
ZLEXCOUNT key min max
```


**Available since** 2.8.9

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
    "type": "string"
  },
  {
    "name": "max",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "Number of elements in the specified score range.",
  "type": "integer",
  "minimum": 0
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
