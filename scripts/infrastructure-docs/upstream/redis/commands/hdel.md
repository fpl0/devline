# HDEL

Deletes one or more fields and their values from a hash. Deletes the hash if no fields remain.

## Syntax

```text
HDEL key field [field ...]
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(N) where N is the number of fields to be removed.

**Arity** -3

**ACL categories** HASH

**Command flags** WRITE, FAST

## History

- 2.4.0 — Accepts multiple `field` arguments.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "field",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "The number of fields that were removed from the hash."
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
