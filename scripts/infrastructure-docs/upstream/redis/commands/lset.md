# LSET

Sets the value of an element in a list by its index.

## Syntax

```text
LSET key index element
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(N) where N is the length of the list. Setting either the first or the last element of the list is O(1).

**Arity** 4

**ACL categories** LIST

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
    "name": "index",
    "type": "integer"
  },
  {
    "name": "element",
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
