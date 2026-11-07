# LRANGE

Returns a range of elements from a list.

## Syntax

```text
LRANGE key start stop
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(S+N) where S is the distance of start offset from HEAD for small lists, from nearest end (HEAD or TAIL) for large lists; and N is the number of elements in the specified range.

**Arity** 4

**ACL categories** LIST

**Command flags** READONLY

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
  "description": "List of elements in the specified range",
  "type": "array",
  "items": {
    "type": "string"
  }
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO",
      "ACCESS"
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
