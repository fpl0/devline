# ZREMRANGEBYLEX

Removes members in a sorted set within a lexicographical range. Deletes the sorted set if all members were removed.

## Syntax

```text
ZREMRANGEBYLEX key min max
```


**Available since** 2.8.9

**Group** sorted_set

**Time complexity** O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements removed by the operation.

**Arity** 4

**ACL categories** SORTEDSET

**Command flags** WRITE

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
  "type": "integer",
  "description": "Number of elements removed."
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
