# HEXISTS

Determines whether a field exists in a hash.

## Syntax

```text
HEXISTS key field
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(1)

**Arity** 3

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
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The hash does not contain the field, or key does not exist.",
      "const": 0
    },
    {
      "description": "The hash contains the field.",
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
      "RO"
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
