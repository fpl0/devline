# SREM

Removes one or more members from a set. Deletes the set if the last member was removed.

## Syntax

```text
SREM key member [member ...]
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(N) where N is the number of members to be removed.

**Arity** -3

**ACL categories** SET

**Command flags** WRITE, FAST

## History

- 2.4.0 — Accepts multiple `member` arguments.

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
  "description": "Number of members that were removed from the set, not including non existing members.",
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
