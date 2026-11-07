# ZINCRBY

Increments the score of a member in a sorted set.

## Syntax

```text
ZINCRBY key increment member
```


**Available since** 1.2.0

**Group** sorted_set

**Time complexity** O(log(N)) where N is the number of elements in the sorted set.

**Arity** 4

**ACL categories** SORTEDSET

**Command flags** WRITE, DENYOOM, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "increment",
    "type": "integer"
  },
  {
    "name": "member",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The new score of `member`",
  "type": "number"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS",
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
  }
]
```
