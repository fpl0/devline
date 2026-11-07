# ZRANK

Returns the index of a member in a sorted set ordered by ascending scores.

## Syntax

```text
ZRANK key member [WITHSCORE]
```


**Available since** 2.0.0

**Group** sorted_set

**Time complexity** O(log(N))

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** READONLY, FAST

## History

- 7.2.0 — Added the optional `WITHSCORE` argument.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "member",
    "type": "string"
  },
  {
    "name": "withscore",
    "token": "WITHSCORE",
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
      "type": "null",
      "description": "Key does not exist or the member does not exist in the sorted set."
    },
    {
      "type": "integer",
      "description": "The rank of the member when 'WITHSCORE' is not used."
    },
    {
      "type": "array",
      "description": "The rank and score of the member when 'WITHSCORE' is used.",
      "minItems": 2,
      "maxItems": 2,
      "items": [
        {
          "type": "integer"
        },
        {
          "type": "number"
        }
      ]
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
