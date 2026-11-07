# AROP

Performs aggregate operations on array elements in a range.

## Syntax

```text
AROP key start end SUM|MIN|MAX|AND|OR|XOR|MATCH value|USED
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(P) where P is visited positions in touched slices (dense scanned slots + sparse entries), with worst-case O(|end-start|+1) and typical case close to O(N), where N is the number of existing elements in range.

**Arity** -5

**ACL categories** ARRAY

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
    "name": "start",
    "type": "integer"
  },
  {
    "name": "end",
    "type": "integer"
  },
  {
    "name": "operation",
    "type": "oneof",
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
        "name": "and",
        "type": "pure-token",
        "token": "AND"
      },
      {
        "name": "or",
        "type": "pure-token",
        "token": "OR"
      },
      {
        "name": "xor",
        "type": "pure-token",
        "token": "XOR"
      },
      {
        "name": "match",
        "type": "block",
        "arguments": [
          {
            "name": "match",
            "type": "pure-token",
            "token": "MATCH"
          },
          {
            "name": "value",
            "type": "string"
          }
        ]
      },
      {
        "name": "used",
        "type": "pure-token",
        "token": "USED"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "Result of the operation.",
      "type": "string"
    },
    {
      "description": "Integer result for MATCH, USED, AND, OR, XOR.",
      "type": "integer"
    },
    {
      "description": "Null if no elements match the operation.",
      "type": "null"
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
