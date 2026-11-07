# SETBIT

Sets or clears the bit at offset of the string value. Creates the key if it doesn't exist.

## Syntax

```text
SETBIT key offset value
```


**Available since** 2.2.0

**Group** bitmap

**Time complexity** O(1)

**Arity** 4

**ACL categories** BITMAP

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "offset",
    "type": "integer"
  },
  {
    "name": "value",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "description": "The original bit value stored at offset.",
  "oneOf": [
    {
      "const": 0
    },
    {
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
      "ACCESS",
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
