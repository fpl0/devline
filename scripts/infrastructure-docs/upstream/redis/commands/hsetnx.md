# HSETNX

Sets the value of a field in a hash only when the field doesn't exist.

## Syntax

```text
HSETNX key field value
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(1)

**Arity** 4

**ACL categories** HASH

**Command flags** WRITE, DENYOOM, FAST

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
  },
  {
    "name": "value",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The field is a new field in the hash and value was set.",
      "const": 0
    },
    {
      "description": "The field already exists in the hash and no operation was performed.",
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
      "RW",
      "INSERT"
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
