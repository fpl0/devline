# MOVE

Moves a key to another database.

## Syntax

```text
MOVE key db
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 3

**ACL categories** KEYSPACE

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "db",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "key was moved",
      "const": 1
    },
    {
      "description": "key wasn't moved",
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
