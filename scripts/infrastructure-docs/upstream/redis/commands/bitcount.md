# BITCOUNT

Counts the number of set bits (population counting) in a string.

## Syntax

```text
BITCOUNT key [start end [BYTE|BIT]]
```


**Available since** 2.6.0

**Group** bitmap

**Time complexity** O(N)

**Arity** -2

**ACL categories** BITMAP

**Command flags** READONLY

## History

- 7.0.0 — Added the `BYTE|BIT` option.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "range",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "start",
        "type": "integer"
      },
      {
        "name": "end",
        "type": "integer"
      },
      {
        "name": "unit",
        "type": "oneof",
        "optional": true,
        "since": "7.0.0",
        "arguments": [
          {
            "name": "byte",
            "type": "pure-token",
            "token": "BYTE"
          },
          {
            "name": "bit",
            "type": "pure-token",
            "token": "BIT"
          }
        ]
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "The number of bits set to 1.",
  "type": "integer",
  "minimum": 0
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
