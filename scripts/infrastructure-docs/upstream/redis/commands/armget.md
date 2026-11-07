# ARMGET

Gets values at multiple indices in an array.

## Syntax

```text
ARMGET key index [index ...]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(N) where N is the number of indices

**Arity** -3

**ACL categories** ARRAY

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
    "name": "index",
    "type": "integer",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "array",
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
