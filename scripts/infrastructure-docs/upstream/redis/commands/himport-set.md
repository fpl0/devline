# HIMPORT SET

Creates a fieldset-based hash from values supplied in the order matching a previously prepared fieldset.

## Syntax

```text
HIMPORT SET key fieldset-name value [value ...]
```


**Available since** 8.10.0

**Group** hash

**Time complexity** O(N) where N is the number of fields in the fieldset.

**Arity** -5

**ACL categories** HASH

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
    "name": "fieldset-name",
    "type": "string"
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
        "pos": 2
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
