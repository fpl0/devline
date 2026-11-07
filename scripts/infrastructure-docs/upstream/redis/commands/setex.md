# SETEX

Sets the string value and expiration time of a key. Creates the key if it doesn't exist.

## Syntax

```text
SETEX key seconds value
```


**Available since** 2.0.0

**Group** string

**Time complexity** O(1)

**Arity** 4

**Deprecated since** 2.6.12

**Replacement** `SET` with the `EX` argument

**ACL categories** STRING

**Command flags** WRITE, DENYOOM

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
    "name": "seconds",
    "type": "integer"
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
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
