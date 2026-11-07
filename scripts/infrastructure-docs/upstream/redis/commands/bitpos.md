# BITPOS

Finds the first set (1) or clear (0) bit in a string.

## Syntax

```text
BITPOS key bit [start [end [BYTE|BIT]]]
```


**Available since** 2.8.7

**Group** bitmap

**Time complexity** O(N)

**Arity** -3

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
    "name": "bit",
    "type": "integer"
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
        "name": "end-unit-block",
        "type": "block",
        "optional": true,
        "arguments": [
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
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "the position of the first bit set to 1 or 0 according to the request",
      "type": "integer",
      "minimum": 0
    },
    {
      "description": "In case the `bit` argument is 1 and the string is empty or composed of just zero bytes",
      "const": -1
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
