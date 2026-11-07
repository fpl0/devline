# UNLINK

Asynchronously deletes one or more keys.

## Syntax

```text
UNLINK key [key ...]
```


**Available since** 4.0.0

**Group** generic

**Time complexity** O(1) for each key removed regardless of its size. Then the command does O(N) work in a different thread in order to reclaim memory, where N is the number of allocations the deleted objects where composed of.

**Arity** -2

**ACL categories** KEYSPACE

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "the number of keys that were unlinked",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RM",
      "DELETE"
    ],
    "begin_search": {
      "index": {
        "pos": 1
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
