# TOUCH

Returns the number of existing keys out of those specified after updating the time they were last accessed.

## Syntax

```text
TOUCH key [key ...]
```


**Available since** 3.2.1

**Group** generic

**Time complexity** O(N) where N is the number of keys that will be touched.

**Arity** -2

**ACL categories** KEYSPACE

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "the number of touched keys",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO"
    ],
    "begin_search": {
      "index": {
        "pos": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
