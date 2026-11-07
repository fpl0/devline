# BRPOP

Removes and returns the last element in a list. Blocks until an element is available otherwise. Deletes the list if the last element was popped.

## Syntax

```text
BRPOP key [key ...] timeout
```


**Available since** 2.0.0

**Group** list

**Time complexity** O(N) where N is the number of provided keys.

**Arity** -3

**ACL categories** LIST

**Command flags** WRITE, BLOCKING

## History

- 6.0.0 — `timeout` is interpreted as a double instead of an integer.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
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
      "description": "No element could be popped and the timeout expired.",
      "type": "null"
    },
    {
      "type": "array",
      "minItems": 2,
      "maxItems": 2,
      "items": [
        {
          "description": "The name of the key where an element was popped ",
          "type": "string"
        },
        {
          "description": "The value of the popped element",
          "type": "string"
        }
      ]
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
        "lastkey": -2,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
