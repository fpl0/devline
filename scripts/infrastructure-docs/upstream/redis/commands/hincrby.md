# HINCRBY

Increments the integer value of a field in a hash by a number. Uses 0 as initial value if the field doesn't exist.

## Syntax

```text
HINCRBY key field increment
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(1)

**Arity** 4

**ACL categories** HASH

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
    "name": "field",
    "type": "string"
  },
  {
    "name": "increment",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "The value of the field after the increment operation."
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
