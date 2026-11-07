# ZMSCORE

Returns the score of one or more members in a sorted set.

## Syntax

```text
ZMSCORE key member [member ...]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(N) where N is the number of members being requested.

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "member",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "minItems": 1,
  "items": {
    "oneOf": [
      {
        "type": "number",
        "description": "The score of the member (a double precision floating point number). In RESP2, this is returned as string."
      },
      {
        "type": "null",
        "description": "Member does not exist in the sorted set."
      }
    ]
  }
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO",
      "ACCESS"
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
