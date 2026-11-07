# XDELEX

Deletes one or multiple entries from the stream.

## Syntax

```text
XDELEX key [KEEPREF|DELREF|ACKED] IDS numids id [id ...]
```


**Available since** 8.2.0

**Group** stream

**Time complexity** O(1) for each single item to delete in the stream, regardless of the stream size.

**Arity** -5

**ACL categories** STREAM

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
    "name": "condition",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "keepref",
        "type": "pure-token",
        "token": "KEEPREF"
      },
      {
        "name": "delref",
        "type": "pure-token",
        "token": "DELREF"
      },
      {
        "name": "acked",
        "type": "pure-token",
        "token": "ACKED"
      }
    ]
  },
  {
    "name": "ids",
    "token": "IDS",
    "type": "block",
    "arguments": [
      {
        "name": "numids",
        "type": "integer"
      },
      {
        "name": "id",
        "type": "string",
        "multiple": true
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "Array of results. Returns an array with -1 for each requested ID if the key does not exist.",
  "type": "array",
  "minItems": 0,
  "maxItems": 4294967295,
  "items": {
    "oneOf": [
      {
        "description": "The id does not exist in the provided stream key.",
        "const": -1
      },
      {
        "description": "Entry was deleted from the stream.",
        "const": 1
      },
      {
        "description": "Entry was not deleted, but there are still dangling references.",
        "const": 2
      }
    ]
  }
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
