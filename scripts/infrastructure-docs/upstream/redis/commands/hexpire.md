# HEXPIRE

Set expiry for hash field using relative time to expire (seconds)

## Syntax

```text
HEXPIRE key seconds [NX|XX|GT|LT] FIELDS numfields field [field ...]
```


**Available since** 7.4.0

**Group** hash

**Time complexity** O(N) where N is the number of specified fields

**Arity** -6

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
    "name": "seconds",
    "type": "integer"
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
  "description": "Array of results. Returns empty array if the key does not exist.",
  "type": "array",
  "minItems": 0,
  "maxItems": 4294967295,
  "items": {
    "oneOf": [
      {
        "description": "The field does not exist.",
        "const": -2
      },
      {
        "description": "Specified NX | XX | GT | LT condition not met",
        "const": 0
      },
      {
        "description": "Expiration time was set or updated.",
        "const": 1
      },
      {
        "description": "Field deleted because the specified expiration time is in the past.",
        "const": 2
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
