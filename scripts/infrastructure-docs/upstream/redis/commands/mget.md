# MGET

Atomically returns the string values of one or more keys.

## Syntax

```text
MGET key [key ...]
```


**Available since** 1.0.0

**Group** string

**Time complexity** O(N) where N is the number of keys to retrieve.

**Arity** -2

**ACL categories** STRING

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "List of values at the specified keys.",
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
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
