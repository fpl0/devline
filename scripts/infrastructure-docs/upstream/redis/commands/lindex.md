# LINDEX

Returns an element from a list by its index.

## Syntax

```text
LINDEX key index
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(N) where N is the number of elements to traverse to get to the element at index. This makes asking for the first or the last element of the list O(1).

**Arity** 3

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
    "name": "index",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "type": "null",
      "description": "Index is out of range"
    },
    {
      "description": "The requested element",
      "type": "string"
    }
  ]
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
