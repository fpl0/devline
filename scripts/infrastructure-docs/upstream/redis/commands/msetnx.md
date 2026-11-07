# MSETNX

Atomically modifies the string values of one or more keys only when all keys don't exist.

## Syntax

```text
MSETNX key value [key value ...]
```


**Available since** 1.0.1

**Group** string

**Time complexity** O(N) where N is the number of keys to set.

**Arity** -3

**ACL categories** STRING

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
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
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "No key was set (at least one key already existed).",
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
      "INSERT"
    ],
    "begin_search": {
      "index": {
        "pos": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": -1,
        "step": 2,
        "limit": 0
      }
    }
  }
]
```
