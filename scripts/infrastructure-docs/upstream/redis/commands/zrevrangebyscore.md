# ZREVRANGEBYSCORE

Returns members in a sorted set within a range of scores in reverse order.

## Syntax

```text
ZREVRANGEBYSCORE key max min [WITHSCORES] [LIMIT offset count]
```


**Available since** 2.2.0

**Group** sorted_set

**Time complexity** O(log(N)+M) with N being the number of elements in the sorted set and M the number of elements being returned. If M is constant (e.g. always asking for the first 10 elements with LIMIT), you can consider it O(log(N)).

**Arity** -4

**Deprecated since** 6.2.0

**Replacement** `ZRANGE` with the `REV` and `BYSCORE` arguments

**ACL categories** SORTEDSET

**Command flags** READONLY

**Documentation flags** DEPRECATED

## History

- 2.1.6 — `min` and `max` can be exclusive.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "max",
    "type": "double"
  },
  {
    "name": "min",
    "type": "double"
  },
  {
    "name": "withscores",
    "token": "WITHSCORES",
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
  "anyOf": [
    {
      "type": "array",
      "description": "List of the elements in the specified score range, as not WITHSCORES",
      "uniqueItems": true,
      "items": {
        "type": "string",
        "description": "Element"
      }
    },
    {
      "type": "array",
      "description": "List of the elements and their scores in the specified score range, as WITHSCORES used",
      "uniqueItems": true,
      "items": {
        "type": "array",
        "description": "Tuple of element and its score",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "type": "string",
            "description": "element"
          },
          {
            "type": "number",
            "description": "score"
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
