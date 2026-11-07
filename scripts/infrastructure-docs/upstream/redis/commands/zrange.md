# ZRANGE

Returns members in a sorted set within a range of indexes.

## Syntax

```text
ZRANGE key start stop [BYSCORE|BYLEX] [REV] [LIMIT offset count] [WITHSCORES]
```


**Available since** 1.2.0

**Group** sorted_set

**Time complexity** O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements returned.

**Arity** -4

**ACL categories** SORTEDSET

**Command flags** READONLY

## History

- 6.2.0 — Added the `REV`, `BYSCORE`, `BYLEX` and `LIMIT` options.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "start",
    "type": "string"
  },
  {
    "name": "stop",
    "type": "string"
  },
  {
    "name": "sortby",
    "type": "oneof",
    "optional": true,
    "since": "6.2.0",
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
    "optional": true,
    "since": "6.2.0"
  },
  {
    "token": "LIMIT",
    "name": "limit",
    "type": "block",
    "optional": true,
    "since": "6.2.0",
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
  },
  {
    "name": "withscores",
    "token": "WITHSCORES",
    "type": "pure-token",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "anyOf": [
    {
      "description": "A list of member elements",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "string"
      }
    },
    {
      "description": "Members and their scores. Returned in case `WITHSCORES` was used. In RESP2 this is returned as a flat array",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "description": "Member",
            "type": "string"
          },
          {
            "description": "Score",
            "type": "number"
          }
        ]
      }
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
