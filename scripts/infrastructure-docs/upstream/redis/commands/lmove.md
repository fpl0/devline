# LMOVE

Returns an element after popping it from one list and pushing it to another. Deletes the list if the last element was moved.

## Syntax

```text
LMOVE source destination LEFT|RIGHT LEFT|RIGHT
```


**Available since** 6.2.0

**Group** list

**Time complexity** O(1)

**Arity** 5

**ACL categories** LIST

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "source",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "destination",
    "type": "key",
    "key_spec_index": 1
  },
  {
    "name": "wherefrom",
    "type": "oneof",
    "arguments": [
      {
        "name": "left",
        "type": "pure-token",
        "token": "LEFT"
      },
      {
        "name": "right",
        "type": "pure-token",
        "token": "RIGHT"
      }
    ]
  },
  {
    "name": "whereto",
    "type": "oneof",
    "arguments": [
      {
        "name": "left",
        "type": "pure-token",
        "token": "LEFT"
      },
      {
        "name": "right",
        "type": "pure-token",
        "token": "RIGHT"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "The element being popped and pushed.",
  "type": "string"
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
  },
  {
    "flags": [
      "RW",
      "INSERT"
    ],
    "begin_search": {
      "index": {
        "pos": 2
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
