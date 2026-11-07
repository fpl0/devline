# EXPIRETIME

Returns the expiration time of a key as a Unix timestamp.

## Syntax

```text
EXPIRETIME key
```


**Available since** 7.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 2

**ACL categories** KEYSPACE

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
  "oneOf": [
    {
      "type": "integer",
      "description": "Expiration Unix timestamp in seconds.",
      "minimum": 0
    },
    {
      "const": -1,
      "description": "The key exists but has no associated expiration time."
    },
    {
      "const": -2,
      "description": "The key does not exist."
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
