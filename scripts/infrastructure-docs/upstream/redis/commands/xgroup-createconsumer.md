# XGROUP CREATECONSUMER

Creates a consumer in a consumer group.

## Syntax

```text
XGROUP CREATECONSUMER key group consumer
```


**Available since** 6.2.0

**Group** stream

**Time complexity** O(1)

**Arity** 5

**ACL categories** STREAM

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
  "description": "The number of created consumers (0 or 1)",
  "oneOf": [
    {
      "const": 1
    },
    {
      "const": 0
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
