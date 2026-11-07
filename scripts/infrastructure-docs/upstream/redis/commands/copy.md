# COPY

Copies the value of a key to a new key.

## Syntax

```text
COPY source destination [DB destination-db] [REPLACE]
```


**Available since** 6.2.0

**Group** generic

**Time complexity** O(N) worst case for collections, where N is the number of nested items. O(1) for string values.

**Arity** -3

**ACL categories** KEYSPACE

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "source",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "destination",
    "type": "key",
    "key_spec_index": 1
  },
  {
    "token": "DB",
    "name": "destination-db",
    "type": "integer",
    "optional": true
  },
  {
    "name": "replace",
    "token": "REPLACE",
    "type": "pure-token",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "source was copied",
      "const": 1
    },
    {
      "description": "source was not copied",
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
