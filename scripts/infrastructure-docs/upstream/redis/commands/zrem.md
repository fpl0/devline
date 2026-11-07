# ZREM

Removes one or more members from a sorted set. Deletes the sorted set if all members were removed.

## Syntax

```text
ZREM key member [member ...]
```


**Available since** 1.2.0

**Group** sorted_set

**Time complexity** O(M*log(N)) with N being the number of elements in the sorted set and M the number of elements to be removed.

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** WRITE, FAST

## History

- 2.4.0 — Accepts multiple elements.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "member",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "The number of members removed from the sorted set, not including non existing members.",
  "type": "integer",
  "minimum": 0
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
