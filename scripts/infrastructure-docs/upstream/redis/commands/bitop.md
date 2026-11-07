# BITOP

Performs bitwise operations on multiple strings, and stores the result.

## Syntax

```text
BITOP AND|OR|XOR|NOT|DIFF|DIFF1|ANDOR|ONE destkey key [key ...]
```


**Available since** 2.6.0

**Group** bitmap

**Time complexity** O(N)

**Arity** -4

**ACL categories** BITMAP

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "operation",
    "type": "oneof",
    "arguments": [
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
        "name": "not",
        "type": "pure-token",
        "token": "NOT"
      },
      {
        "name": "diff",
        "type": "pure-token",
        "token": "DIFF"
      },
      {
        "name": "diff1",
        "type": "pure-token",
        "token": "DIFF1"
      },
      {
        "name": "andor",
        "type": "pure-token",
        "token": "ANDOR"
      },
      {
        "name": "one",
        "type": "pure-token",
        "token": "ONE"
      }
    ]
  },
  {
    "name": "destkey",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 1,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "the size of the string stored in the destination key, that is equal to the size of the longest input string",
  "type": "integer",
  "minimum": 0
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
        "pos": 2
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
        "pos": 3
      }
    },
    "find_keys": {
      "range": {
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
