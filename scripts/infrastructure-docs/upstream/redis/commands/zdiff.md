# ZDIFF

Returns the difference between multiple sorted sets.

## Syntax

```text
ZDIFF numkeys key [key ...] [WITHSCORES]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(L + (N-K)log(N)) worst case where L is the total number of elements in all the sets, N is the size of the first set, and K is the size of the result set.

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** READONLY

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
      "description": "A list of members. Returned in case `WITHSCORES` was not used.",
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    {
      "description": "Members and their scores. Returned in case `WITHSCORES` was used. In RESP2 this is returned as a flat array",
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
