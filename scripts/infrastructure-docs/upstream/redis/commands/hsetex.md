# HSETEX

Set the value of one or more fields of a given hash key, and optionally set their expiration.

## Syntax

```text
HSETEX key [FNX|FXX] [EX seconds|PX milliseconds|EXAT unix-time-seconds|PXAT unix-time-milliseconds|KEEPTTL] FIELDS numfields field value [field value ...]
```


**Available since** 8.0.0

**Group** hash

**Time complexity** O(N) where N is the number of fields being set.

**Arity** -6

**ACL categories** HASH

**Command flags** WRITE, DENYOOM, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "condition",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "fnx",
        "type": "pure-token",
        "token": "FNX"
      },
      {
        "name": "fxx",
        "type": "pure-token",
        "token": "FXX"
      }
    ]
  },
  {
    "name": "expiration",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "seconds",
        "type": "integer",
        "token": "EX"
      },
      {
        "name": "milliseconds",
        "type": "integer",
        "token": "PX"
      },
      {
        "name": "unix-time-seconds",
        "type": "unix-time",
        "token": "EXAT"
      },
      {
        "name": "unix-time-milliseconds",
        "type": "unix-time",
        "token": "PXAT"
      },
      {
        "name": "keepttl",
        "type": "pure-token",
        "token": "KEEPTTL"
      }
    ]
  },
  {
    "name": "fields",
    "token": "FIELDS",
    "type": "block",
    "arguments": [
      {
        "name": "numfields",
        "type": "integer"
      },
      {
        "name": "data",
        "type": "block",
        "multiple": true,
        "arguments": [
          {
            "name": "field",
            "type": "string"
          },
          {
            "name": "value",
            "type": "string"
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
      "description": "No field was set (due to FXX or FNX flags).",
      "const": 0
    },
    {
      "description": "All the fields were set.",
      "const": 1
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
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
  }
]
```
