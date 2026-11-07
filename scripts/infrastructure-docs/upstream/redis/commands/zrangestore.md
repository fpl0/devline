# ZRANGESTORE

Stores a range of members from sorted set in a key.

## Syntax

```text
ZRANGESTORE dst src min max [BYSCORE|BYLEX] [REV] [LIMIT offset count]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements stored into the destination key.

**Arity** -5

**ACL categories** SORTEDSET

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "dst",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "src",
    "type": "key",
    "key_spec_index": 1
  },
  {
    "name": "min",
    "type": "string"
  },
  {
    "name": "max",
    "type": "string"
  },
  {
    "name": "sortby",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "byscore",
        "type": "pure-token",
        "token": "BYSCORE"
      },
      {
        "name": "bylex",
        "type": "pure-token",
        "token": "BYLEX"
      }
    ]
  },
  {
    "name": "rev",
    "token": "REV",
    "type": "pure-token",
    "optional": true
  },
  {
    "token": "LIMIT",
    "name": "limit",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "offset",
        "type": "integer"
      },
      {
        "name": "count",
        "type": "integer"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "Number of elements in the resulting sorted set."
}
```


## Key specifications

```json
[
  {
    "flags": [
      "OW",
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
  },
  {
    "flags": [
      "RO",
      "ACCESS"
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
