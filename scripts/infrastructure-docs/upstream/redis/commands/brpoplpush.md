# BRPOPLPUSH

Pops an element from a list, pushes it to another list and returns it. Block until an element is available otherwise. Deletes the list if the last element was popped.

## Syntax

```text
BRPOPLPUSH source destination timeout
```


**Available since** 2.2.0

**Group** list

**Time complexity** O(1)

**Arity** 4

**Deprecated since** 6.2.0

**Replacement** `BLMOVE` with the `RIGHT` and `LEFT` arguments

**ACL categories** LIST

**Command flags** WRITE, DENYOOM, BLOCKING

**Documentation flags** DEPRECATED

## History

- 6.0.0 — `timeout` is interpreted as a double instead of an integer.

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
      "type": "string",
      "description": "The element being popped from source and pushed to destination."
    },
    {
      "type": "null",
      "description": "Timeout is reached."
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
