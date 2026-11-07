# XGROUP DESTROY

Destroys a consumer group.

## Syntax

```text
XGROUP DESTROY key group
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(N) where N is the number of entries in the group's pending entries list (PEL).

**Arity** 4

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
  }
]
```


## Reply schema

```json
{
  "description": "The number of destroyed consumer groups (0 or 1)",
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
