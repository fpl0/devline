# LPOP

Returns the first elements in a list after removing it. Deletes the list if the last element was popped.

## Syntax

```text
LPOP key [count]
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(N) where N is the number of elements returned

**Arity** -2

**ACL categories** LIST

**Command flags** WRITE, FAST

## History

- 6.2.0 — Added the `count` argument.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "count",
    "type": "integer",
    "optional": true,
    "since": "6.2.0"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "Key does not exist.",
      "type": "null"
    },
    {
      "description": "In case `count` argument was not given, the value of the first element.",
      "type": "string"
    },
    {
      "description": "In case `count` argument was given, a list of popped elements",
      "type": "array",
      "items": {
        "type": "string"
      }
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
  }
]
```
