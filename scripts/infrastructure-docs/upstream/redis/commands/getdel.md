# GETDEL

Returns the string value of a key after deleting the key.

## Syntax

```text
GETDEL key
```


**Available since** 6.2.0

**Group** string

**Time complexity** O(1)

**Arity** 2

**ACL categories** STRING

**Command flags** WRITE, FAST

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
  "oneOf": [
    {
      "description": "The value of the key.",
      "type": "string"
    },
    {
      "description": "The key does not exist.",
      "type": "null"
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS",
      "DELETE"
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
