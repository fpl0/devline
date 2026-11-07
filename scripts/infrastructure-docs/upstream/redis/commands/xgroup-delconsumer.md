# XGROUP DELCONSUMER

Deletes a consumer from a consumer group.

## Syntax

```text
XGROUP DELCONSUMER key group consumer
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** 5

**ACL categories** STREAM

**Command flags** WRITE

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "group",
    "type": "string"
  },
  {
    "name": "consumer",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The number of pending messages that were yet associated with such a consumer",
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
