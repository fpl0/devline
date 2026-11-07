# HPEXPIRETIME

Returns the expiration time of a hash field as a Unix timestamp, in msec.

## Syntax

```text
HPEXPIRETIME key FIELDS numfields field [field ...]
```


**Available since** 7.4.0

**Group** hash

**Time complexity** O(N) where N is the number of specified fields

**Arity** -5

**ACL categories** HASH

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
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
        "description": "The field exists but has no associated expire.",
        "const": -1
      },
      {
        "description": "Expiration Unix timestamp in milliseconds.",
        "type": "integer",
        "minimum": 1
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
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
