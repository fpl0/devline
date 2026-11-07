# ARGETRANGE

Gets values in a range of indices.

## Syntax

```text
ARGETRANGE key start end
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(N) where N is the range length

**Arity** 4

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
