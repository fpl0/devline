# PERSIST

Removes the expiration time of a key.

## Syntax

```text
PERSIST key
```


**Available since** 2.2.0

**Group** generic

**Time complexity** O(1)

**Arity** 2

**ACL categories** KEYSPACE

**Command flags** WRITE, FAST

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
      "const": 0,
      "description": "Key does not exist or does not have an associated timeout."
    },
    {
      "const": 1,
      "description": "The timeout has been removed."
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
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
