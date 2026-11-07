# LTRIM

Removes elements from both ends a list. Deletes the list if all elements were trimmed.

## Syntax

```text
LTRIM key start stop
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(N) where N is the number of elements to be removed by the operation.

**Arity** 4

**ACL categories** LIST

**Command flags** WRITE

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "start",
    "type": "integer"
  },
  {
    "name": "stop",
    "type": "integer"
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
