# ZREVRANGE

Returns members in a sorted set within a range of indexes in reverse order.

## Syntax

```text
ZREVRANGE key start stop [WITHSCORES]
```


**Available since** 1.2.0

**Group** sorted_set

**Time complexity** O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements returned.

**Arity** -4

**Deprecated since** 6.2.0

**Replacement** `ZRANGE` with the `REV` argument

**ACL categories** SORTEDSET

**Command flags** READONLY

**Documentation flags** DEPRECATED

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
    "type": "integer"
  },
  {
    "name": "stop",
    "type": "integer"
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
      "description": "List of member elements.",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "string"
      }
    },
    {
      "description": "List of the members and their scores. Returned in case `WITHSCORES` was used.",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "description": "member",
            "type": "string"
          },
          {
            "description": "score",
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
