# HGET

Returns the value of a field in a hash.

## Syntax

```text
HGET key field
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
      "description": "The value associated with the field.",
      "type": "string"
    },
    {
      "description": "If the field is not present in the hash or key does not exist.",
      "type": "null"
    }
  ]
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
