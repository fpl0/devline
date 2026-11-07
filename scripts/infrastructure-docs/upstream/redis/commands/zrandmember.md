# ZRANDMEMBER

Returns one or more random members from a sorted set.

## Syntax

```text
ZRANDMEMBER key [count [WITHSCORES]]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(N) where N is the number of members returned

**Arity** -2

**ACL categories** SORTEDSET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "options",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "count",
        "type": "integer"
      },
      {
        "name": "withscores",
        "token": "WITHSCORES",
        "type": "pure-token",
        "optional": true
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
      "type": "null",
      "description": "Key does not exist."
    },
    {
      "type": "string",
      "description": "Randomly selected element when 'COUNT' is not used."
    },
    {
      "type": "array",
      "description": "Randomly selected elements when 'COUNT' is used.",
      "items": {
        "type": "string"
      }
    },
    {
      "type": "array",
      "description": "Randomly selected elements when 'COUNT' and 'WITHSCORES' modifiers are used.",
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "type": "string",
            "description": "Element."
          },
          {
            "type": "number",
            "description": "Score."
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
