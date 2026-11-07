# XDEL

Returns the number of messages after removing them from a stream.

## Syntax

```text
XDEL key id [id ...]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1) for each single item to delete in the stream, regardless of the stream size.

**Arity** -3

**ACL categories** STREAM

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "ID",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "The number of entries actually deleted",
  "type": "integer",
  "minimum": 0
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
