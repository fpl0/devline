# ARSCAN

Iterates existing elements in a range, returning index-value pairs.

## Syntax

```text
ARSCAN key start end [LIMIT limit]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(P) where P is visited positions in touched slices (dense scanned slots + sparse entries), with worst-case O(|end-start|+1) and typical case close to O(N), where N is the number of existing elements in range.

**Arity** -4

**ACL categories** ARRAY

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "start",
    "type": "integer"
  },
  {
    "name": "end",
    "type": "integer"
  },
  {
    "name": "limit",
    "token": "LIMIT",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "Array of [index, value] pairs.",
  "type": "array",
  "items": {
    "type": "array",
    "minItems": 2,
    "maxItems": 2,
    "items": [
      {
        "type": "integer",
        "description": "Index of existing element"
      },
      {
        "type": "string",
        "description": "Value at that index"
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
