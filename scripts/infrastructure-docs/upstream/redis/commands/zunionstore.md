# ZUNIONSTORE

Stores the union of multiple sorted sets in a key.

## Syntax

```text
ZUNIONSTORE destination numkeys key [key ...] [WEIGHTS weight [weight ...]] [AGGREGATE SUM|MIN|MAX|COUNT]
```


**Available since** 2.0.0

**Group** sorted_set

**Time complexity** O(N)+O(M log(M)) with N being the sum of the sizes of the input sorted sets, and M being the number of elements in the resulting sorted set.

**Arity** -4

**ACL categories** SORTEDSET

**Command flags** WRITE, DENYOOM

## History

- 8.8.0 — Added `COUNT` aggregate option.

## Arguments

```json
[
  {
    "name": "destination",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 1,
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
  }
]
```


## Reply schema

```json
{
  "description": "The number of elements in the resulting sorted set.",
  "type": "integer"
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
      "keynum": {
        "keynumidx": 0,
        "firstkey": 1,
        "step": 1
      }
    }
  }
]
```
