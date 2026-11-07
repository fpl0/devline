# ARGET

Gets the value at an index in an array.

## Syntax

```text
ARGET key index
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(1)

**Arity** 3

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
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The value at the given index.",
      "type": "string"
    },
    {
      "description": "Null reply if key or index does not exist.",
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
