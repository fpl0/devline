# SADD

Adds one or more members to a set. Creates the key if it doesn't exist.

## Syntax

```text
SADD key member [member ...]
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(1) for each element added, so O(N) to add N elements when the command is called with multiple arguments.

**Arity** -3

**ACL categories** SET

**Command flags** WRITE, DENYOOM, FAST

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
  "description": "Number of elements that were added to the set, not including all the elements already present in the set.",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "INSERT"
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
