# DECRBY

Decrements a number from the integer value of a key. Uses 0 as initial value if the key doesn't exist.

## Syntax

```text
DECRBY key decrement
```


**Available since** 1.0.0

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
    "name": "decrement",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "The value of the key after decrementing it."
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
