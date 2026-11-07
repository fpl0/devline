# INCRBYFLOAT

Increment the floating point value of a key by a number. Uses 0 as initial value if the key doesn't exist.

## Syntax

```text
INCRBYFLOAT key increment
```


**Available since** 2.6.0

**Group** string

**Time complexity** O(1)

**Arity** 3

**ACL categories** STRING

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
    "name": "increment",
    "type": "double"
  }
]
```


## Reply schema

```json
{
  "type": "string",
  "description": "The value of the key after incrementing it."
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
