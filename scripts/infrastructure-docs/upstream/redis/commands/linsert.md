# LINSERT

Inserts an element before or after another element in a list.

## Syntax

```text
LINSERT key BEFORE|AFTER pivot element
```


**Available since** 2.2.0

**Group** list

**Time complexity** O(N) where N is the number of elements to traverse before seeing the value pivot. This means that inserting somewhere on the left end on the list (head) can be considered O(1) and inserting somewhere on the right end (tail) is O(N).

**Arity** 5

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
    "name": "where",
    "type": "oneof",
    "arguments": [
      {
        "name": "before",
        "type": "pure-token",
        "token": "BEFORE"
      },
      {
        "name": "after",
        "type": "pure-token",
        "token": "AFTER"
      }
    ]
  },
  {
    "name": "pivot",
    "type": "string"
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
  "oneOf": [
    {
      "description": "List length after a successful insert operation.",
      "type": "integer",
      "minimum": 1
    },
    {
      "description": "in case key doesn't exist.",
      "const": 0
    },
    {
      "description": "when the pivot wasn't found.",
      "const": -1
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
