# HGETEX

Get the value of one or more fields of a given hash key, and optionally set their expiration.

## Syntax

```text
HGETEX key [EX seconds|PX milliseconds|EXAT unix-time-seconds|PXAT unix-time-milliseconds|PERSIST] FIELDS numfields field [field ...]
```


**Available since** 8.0.0

**Group** hash

**Time complexity** O(N) where N is the number of specified fields

**Arity** -5

**ACL categories** HASH

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
        "name": "field",
        "type": "string",
        "multiple": true
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "List of values associated with the given fields, in the same order as they are requested.",
  "type": "array",
  "minItems": 1,
  "items": {
    "oneOf": [
      {
        "type": "string"
      },
      {
        "type": "null"
      }
    ]
  }
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
