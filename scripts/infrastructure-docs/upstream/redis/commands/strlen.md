# STRLEN

Returns the length of a string value.

## Syntax

```text
STRLEN key
```


**Available since** 2.2.0

**Group** string

**Time complexity** O(1)

**Arity** 2

**ACL categories** STRING

**Command flags** READONLY, FAST

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
  "description": "The length of the string value stored at key, or 0 when key does not exist.",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO"
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
