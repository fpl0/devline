# BZPOPMAX

Removes and returns the member with the highest score from one or more sorted sets. Blocks until a member available otherwise.  Deletes the sorted set if the last element was popped.

## Syntax

```text
BZPOPMAX key [key ...] timeout
```


**Available since** 5.0.0

**Group** sorted_set

**Time complexity** O(log(N)) with N being the number of elements in the sorted set.

**Arity** -3

**ACL categories** SORTEDSET

**Command flags** WRITE, FAST, BLOCKING

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
      "description": "Timeout reached and no elements were popped.",
      "type": "null"
    },
    {
      "description": "The keyname, popped member, and its score.",
      "type": "array",
      "minItems": 3,
      "maxItems": 3,
      "items": [
        {
          "description": "Keyname",
          "type": "string"
        },
        {
          "description": "Member",
          "type": "string"
        },
        {
          "description": "Score",
          "type": "number"
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
