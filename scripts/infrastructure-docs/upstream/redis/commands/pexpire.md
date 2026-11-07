# PEXPIRE

Sets the expiration time of a key in milliseconds.

## Syntax

```text
PEXPIRE key milliseconds [NX|XX|GT|LT]
```


**Available since** 2.6.0

**Group** generic

**Time complexity** O(1)

**Arity** -3

**ACL categories** KEYSPACE

**Command flags** WRITE, FAST

## History

- 7.0.0 — Added options: `NX`, `XX`, `GT` and `LT`.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "milliseconds",
    "type": "integer"
  },
  {
    "name": "condition",
    "type": "oneof",
    "optional": true,
    "since": "7.0.0",
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
      },
      {
        "name": "gt",
        "type": "pure-token",
        "token": "GT"
      },
      {
        "name": "lt",
        "type": "pure-token",
        "token": "LT"
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
      "const": 0,
      "description": "The timeout was not set. e.g. key doesn't exist, or operation skipped due to the provided arguments."
    },
    {
      "const": 1,
      "description": "The timeout was set."
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
