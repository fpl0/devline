# SETNX

Set the string value of a key only when the key doesn't exist.

## Syntax

```text
SETNX key value
```


**Available since** 1.0.0

**Group** string

**Time complexity** O(1)

**Arity** 3

**Deprecated since** 2.6.12

**Replacement** `SET` with the `NX` argument

**ACL categories** STRING

**Command flags** WRITE, DENYOOM, FAST

**Documentation flags** DEPRECATED

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
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The key was set.",
      "const": 0
    },
    {
      "description": "The key was not set.",
      "const": 1
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "OW",
      "INSERT"
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
