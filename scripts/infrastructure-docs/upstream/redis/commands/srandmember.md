# SRANDMEMBER

Get one or multiple random members from a set

## Syntax

```text
SRANDMEMBER key [count]
```


**Available since** 1.0.0

**Group** set

**Time complexity** Without the count argument O(1), otherwise O(N) where N is the absolute value of the passed count.

**Arity** -2

**ACL categories** SET

**Command flags** READONLY

## History

- 2.6.0 — Added the optional `count` argument.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "count",
    "type": "integer",
    "optional": true,
    "since": "2.6.0"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "In case `count` is not given and key doesn't exist",
      "type": "null"
    },
    {
      "description": "In case `count` is not given, randomly selected element",
      "type": "string"
    },
    {
      "description": "In case `count` is given, an array of elements",
      "type": "array",
      "items": {
        "type": "string"
      },
      "minItems": 1
    },
    {
      "description": "In case `count` is given and key doesn't exist",
      "type": "array",
      "maxItems": 0
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
