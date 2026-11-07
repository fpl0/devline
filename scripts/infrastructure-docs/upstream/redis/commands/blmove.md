# BLMOVE

Pops an element from a list, pushes it to another list and returns it. Blocks until an element is available otherwise. Deletes the list if the last element was moved.

## Syntax

```text
BLMOVE source destination LEFT|RIGHT LEFT|RIGHT timeout
```


**Available since** 6.2.0

**Group** list

**Time complexity** O(1)

**Arity** 6

**ACL categories** LIST

**Command flags** WRITE, DENYOOM, BLOCKING

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
  },
  {
    "name": "timeout",
    "type": "double"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The popped element.",
      "type": "string"
    },
    {
      "description": "Operation timed-out",
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
