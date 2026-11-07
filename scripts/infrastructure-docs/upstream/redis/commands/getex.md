# GETEX

Returns the string value of a key after setting its expiration time.

## Syntax

```text
GETEX key [EX seconds|PX milliseconds|EXAT unix-time-seconds|PXAT unix-time-milliseconds|PERSIST]
```


**Available since** 6.2.0

**Group** string

**Time complexity** O(1)

**Arity** -2

**ACL categories** STRING

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
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
        "name": "persist",
        "type": "pure-token",
        "token": "PERSIST"
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
      "description": "The value of the key.",
      "type": "string"
    },
    {
      "description": "Key does not exist.",
      "type": "null"
    }
  ]
}
```


## Key specifications

```json
[
  {
    "notes": "RW and UPDATE because it changes the TTL",
    "flags": [
      "RW",
      "ACCESS",
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
