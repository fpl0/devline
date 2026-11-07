# RENAMENX

Renames a key only when the target key name doesn't exist.

## Syntax

```text
RENAMENX key newkey
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 3

**ACL categories** KEYSPACE

**Command flags** WRITE, FAST

## History

- 3.2.0 — The command no longer returns an error when source and destination names are the same.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "newkey",
    "type": "key",
    "key_spec_index": 1
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "key was renamed to newkey",
      "const": 1
    },
    {
      "description": "new key already exists",
      "const": 0
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
      "ACCESS",
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
  },
  {
    "flags": [
      "OW",
      "INSERT"
    ],
    "begin_search": {
      "index": {
        "pos": 2
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
