# MSETEX

Atomically sets multiple string keys with a shared expiration in a single operation. Supports flexible argument parsing where condition and expiration flags can appear in any order.

## Syntax

```text
MSETEX numkeys key value [key value ...] [NX|XX] [EX seconds|PX milliseconds|EXAT unix-time-seconds|PXAT unix-time-milliseconds|KEEPTTL]
```


**Available since** 8.4.0

**Group** string

**Time complexity** O(N) where N is the number of keys to set.

**Arity** -4

**ACL categories** STRING

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "data",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "key",
        "type": "key",
        "key_spec_index": 0
      },
      {
        "name": "value",
        "type": "string"
      }
    ]
  },
  {
    "name": "condition",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "nx",
        "type": "pure-token",
        "token": "NX"
      },
      {
        "name": "xx",
        "type": "pure-token",
        "token": "XX"
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
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "No key was set (at least one key failed).",
      "const": 0
    },
    {
      "description": "All the keys were set.",
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
      "OW",
      "UPDATE"
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
        "step": 2
      }
    }
  }
]
```
