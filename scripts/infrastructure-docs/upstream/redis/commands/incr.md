# INCR

Increments the integer value of a key by one. Uses 0 as initial value if the key doesn't exist.

## Syntax

```text
INCR key
```


**Available since** 1.0.0

**Group** string

**Time complexity** O(1)

**Arity** 2

**ACL categories** STRING

**Command flags** WRITE, DENYOOM, FAST

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
  "description": "The value of key after the increment",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS",
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
