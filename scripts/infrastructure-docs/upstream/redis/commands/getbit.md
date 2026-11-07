# GETBIT

Returns a bit value by offset.

## Syntax

```text
GETBIT key offset
```


**Available since** 2.2.0

**Group** bitmap

**Time complexity** O(1)

**Arity** 3

**ACL categories** BITMAP

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
    "name": "offset",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "description": "The bit value stored at offset.",
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
