# ZINTER

Returns the intersect of multiple sorted sets.

## Syntax

```text
ZINTER numkeys key [key ...] [WEIGHTS weight [weight ...]] [AGGREGATE SUM|MIN|MAX|COUNT] [WITHSCORES]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(N*K)+O(M*log(M)) worst case with N being the smallest input sorted set, K being the number of input sorted sets and M being the number of elements in the resulting sorted set.

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** READONLY

## History

- 8.8.0 — Added `COUNT` aggregate option.

## Arguments

```json
[
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  },
  {
    "token": "WEIGHTS",
    "name": "weight",
    "type": "integer",
    "optional": true,
    "multiple": true
  },
  {
    "token": "AGGREGATE",
    "name": "aggregate",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "sum",
        "type": "pure-token",
        "token": "SUM"
      },
      {
        "name": "min",
        "type": "pure-token",
        "token": "MIN"
      },
      {
        "name": "max",
        "type": "pure-token",
        "token": "MAX"
      },
      {
        "name": "count",
        "type": "pure-token",
        "token": "COUNT",
        "since": "8.8.0"
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
      "description": "Result of intersection, containing only the member names. Returned in case `WITHSCORES` was not used.",
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    {
      "description": "Result of intersection, containing members and their scores. Returned in case `WITHSCORES` was used. In RESP2 this is returned as a flat array",
      "type": "array",
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
      "keynum": {
        "keynumidx": 0,
        "firstkey": 1,
        "step": 1
      }
    }
  }
]
```
