# ZSCORE

Returns the score of a member in a sorted set.

## Syntax

```text
ZSCORE key member
```


**Available since** 1.2.0

**Group** sorted_set

**Time complexity** O(1)

**Arity** 3

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
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "type": "number",
      "description": "The score of the member (a double precision floating point number). In RESP2, this is returned as string."
    },
    {
      "type": "null",
      "description": "Member does not exist in the sorted set, or key does not exist."
    }
  ]
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
