# ARRING

Inserts values into a ring buffer of specified size, wrapping and truncating as needed.

## Syntax

```text
ARRING key size value [value ...]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(M) normally, O(N+M) on ring resize, where N is the maximum of the old and new ring size and M is the number of inserted values

**Arity** -4

**ACL categories** ARRAY

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
    "name": "size",
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
