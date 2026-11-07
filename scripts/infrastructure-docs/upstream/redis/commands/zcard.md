# ZCARD

Returns the number of members in a sorted set.

## Syntax

```text
ZCARD key
```


**Available since** 1.2.0

**Group** sorted_set

**Time complexity** O(1)

**Arity** 2

**ACL categories** SORTEDSET

**Command flags** READONLY, FAST

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
  "description": "The cardinality (number of elements) of the sorted set, or 0 if key does not exist",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO"
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
