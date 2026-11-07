# ARSET

Sets one or more contiguous values starting at an index in an array.

## Syntax

```text
ARSET key index value [value ...]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(N) where N is the number of values

**Arity** -4

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
    "name": "index",
    "type": "integer"
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
  "description": "Number of new slots that were set (previously empty).",
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
