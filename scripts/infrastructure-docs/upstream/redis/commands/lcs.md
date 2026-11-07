# LCS

Finds the longest common substring.

## Syntax

```text
LCS key1 key2 [LEN] [IDX] [MINMATCHLEN min-match-len] [WITHMATCHLEN]
```


**Available since** 7.0.0

**Group** string

**Time complexity** O(N*M) where N and M are the lengths of s1 and s2, respectively

**Arity** -3

**ACL categories** STRING

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key1",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "key2",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "len",
    "token": "LEN",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "idx",
    "token": "IDX",
    "type": "pure-token",
    "optional": true
  },
  {
    "token": "MINMATCHLEN",
    "name": "min-match-len",
    "type": "integer",
    "optional": true
  },
  {
    "name": "withmatchlen",
    "token": "WITHMATCHLEN",
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
      "type": "string",
      "description": "The longest common subsequence."
    },
    {
      "type": "integer",
      "description": "The length of the longest common subsequence when 'LEN' is given."
    },
    {
      "type": "object",
      "description": "Array with the LCS length and all the ranges in both the strings when 'IDX' is given. In RESP2 this is returned as a flat array",
      "additionalProperties": false,
      "properties": {
        "matches": {
          "type": "array",
          "items": {
            "type": "array",
            "minItems": 2,
            "maxItems": 3,
            "items": [
              {
                "type": "array",
                "description": "Matched range in the first string.",
                "minItems": 2,
                "maxItems": 2,
                "items": {
                  "type": "integer"
                }
              },
              {
                "type": "array",
                "description": "Matched range in the second string.",
                "minItems": 2,
                "maxItems": 2,
                "items": {
                  "type": "integer"
                }
              }
            ],
            "additionalItems": {
              "type": "integer",
              "description": "The length of the match when 'WITHMATCHLEN' is given."
            }
          }
        },
        "len": {
          "type": "integer",
          "description": "Length of the longest common subsequence."
        }
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
        "lastkey": 1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
