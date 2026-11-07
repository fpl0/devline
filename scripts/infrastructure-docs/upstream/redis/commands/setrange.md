# SETRANGE

Overwrites a part of a string value with another by an offset. Creates the key if it doesn't exist.

## Syntax

```text
SETRANGE key offset value
```


**Available since** 2.2.0

**Group** string

**Time complexity** O(1), not counting the time taken to copy the new string in place. Usually, this string is very small so the amortized complexity is O(1). Otherwise, complexity is O(M) with M being the length of the value argument.

**Arity** 4

**ACL categories** STRING

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
    "name": "offset",
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
  "description": "Length of the string after it was modified by the command.",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
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
