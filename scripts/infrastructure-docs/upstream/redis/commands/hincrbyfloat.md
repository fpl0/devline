# HINCRBYFLOAT

Increments the floating point value of a field by a number. Uses 0 as initial value if the field doesn't exist.

## Syntax

```text
HINCRBYFLOAT key field increment
```


**Available since** 2.6.0

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
    "type": "double"
  }
]
```


## Reply schema

```json
{
  "type": "string",
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
