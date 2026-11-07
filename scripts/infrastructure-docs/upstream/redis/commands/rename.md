# RENAME

Renames a key and overwrites the destination.

## Syntax

```text
RENAME key newkey
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 3

**ACL categories** KEYSPACE

**Command flags** WRITE

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
  "const": "OK"
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
      "UPDATE"
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
