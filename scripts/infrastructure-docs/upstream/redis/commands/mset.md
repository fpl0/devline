# MSET

Atomically creates or modifies the string values of one or more keys.

## Syntax

```text
MSET key value [key value ...]
```


**Available since** 1.0.1

**Group** string

**Time complexity** O(N) where N is the number of keys to set.

**Arity** -3

**ACL categories** STRING

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "data",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "key",
        "type": "key",
        "key_spec_index": 0
      },
      {
        "name": "value",
        "type": "string"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "OW",
      "UPDATE"
    ],
    "begin_search": {
      "index": {
        "pos": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": -1,
        "step": 2,
        "limit": 0
      }
    }
  }
]
```
