# HVALS

Returns all values in a hash.

## Syntax

```text
HVALS key
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(N) where N is the size of the hash.

**Arity** 2

**ACL categories** HASH

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "List of values in the hash, or an empty list when the key does not exist.",
  "items": {
    "type": "string"
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
