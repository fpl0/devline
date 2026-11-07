# HGETDEL

Returns the value of a field and deletes it from the hash.

## Syntax

```text
HGETDEL key FIELDS numfields field [field ...]
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
    "flags": [
      "RW",
      "ACCESS",
      "DELETE"
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
