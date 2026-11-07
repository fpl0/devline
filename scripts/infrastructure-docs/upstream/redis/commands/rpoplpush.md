# RPOPLPUSH

Returns the last element of a list after removing and pushing it to another list. Deletes the list if the last element was popped.

## Syntax

```text
RPOPLPUSH source destination
```


**Available since** 1.2.0

**Group** list

**Time complexity** O(1)

**Arity** 3

**Deprecated since** 6.2.0

**Replacement** `LMOVE` with the `RIGHT` and `LEFT` arguments

**ACL categories** LIST

**Command flags** WRITE, DENYOOM

**Documentation flags** DEPRECATED

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
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "type": "string",
      "description": "The element being popped and pushed."
    },
    {
      "type": "null",
      "description": "Source list is empty."
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
