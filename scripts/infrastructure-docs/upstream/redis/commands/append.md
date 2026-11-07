# APPEND

Appends a string to the value of a key. Creates the key if it doesn't exist.

## Syntax

```text
APPEND key value
```


**Available since** 2.0.0

**Group** string

**Time complexity** O(1). The amortized time complexity is O(1) assuming the appended value is small and the already present value is of any size, since the dynamic string library used by Redis will double the free space available on every reallocation.

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
    "name": "value",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "The length of the string after the append operation."
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
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
