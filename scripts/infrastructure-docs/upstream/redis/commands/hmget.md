# HMGET

Returns the values of all fields in a hash.

## Syntax

```text
HMGET key field [field ...]
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(N) where N is the number of fields being requested.

**Arity** -3

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
    "name": "field",
    "type": "string",
    "multiple": true
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
