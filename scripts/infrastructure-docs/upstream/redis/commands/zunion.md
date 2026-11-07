# ZUNION

Returns the union of multiple sorted sets.

## Syntax

```text
ZUNION numkeys key [key ...] [WEIGHTS weight [weight ...]] [AGGREGATE SUM|MIN|MAX|COUNT] [WITHSCORES]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(N)+O(M*log(M)) with N being the sum of the sizes of the input sorted sets, and M being the number of elements in the resulting sorted set.

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
      "description": "The result of union when 'WITHSCORES' is not used.",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "string"
      }
    },
    {
      "description": "The result of union when 'WITHSCORES' is used.",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "type": "string"
          },
          {
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
