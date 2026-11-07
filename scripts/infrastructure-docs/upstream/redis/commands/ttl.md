# TTL

Returns the expiration time in seconds of a key.

## Syntax

```text
TTL key
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 2

**ACL categories** KEYSPACE

**Command flags** READONLY, FAST

## History

- 2.8.0 — Added the -2 reply.

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
  "oneOf": [
    {
      "description": "TTL in seconds.",
      "type": "integer",
      "minimum": 0
    },
    {
      "description": "The key exists but has no associated expire.",
      "const": -1
    },
    {
      "description": "The key does not exist.",
      "const": -2
    }
  ]
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
