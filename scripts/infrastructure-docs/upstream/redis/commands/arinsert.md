# ARINSERT

Inserts one or more values at consecutive indices.

## Syntax

```text
ARINSERT key value [value ...]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(N) where N is the number of values

**Arity** -3

**ACL categories** ARRAY

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
    "name": "value",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "The last index where a value was inserted.",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
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
