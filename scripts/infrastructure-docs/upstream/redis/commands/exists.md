# EXISTS

Determines whether one or more keys exist.

## Syntax

```text
EXISTS key [key ...]
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(N) where N is the number of keys to check.

**Arity** -2

**ACL categories** KEYSPACE

**Command flags** READONLY, FAST

## History

- 3.0.3 — Accepts multiple `key` arguments.

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
  "description": "Number of keys that exist from those specified as arguments.",
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
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
