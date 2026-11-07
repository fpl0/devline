# LPOS

Returns the index of matching elements in a list.

## Syntax

```text
LPOS key element [RANK rank] [COUNT num-matches] [MAXLEN len]
```


**Available since** 6.0.6

**Group** list

**Time complexity** O(N) where N is the number of elements in the list, for the average case. When searching for elements near the head or the tail of the list, or when the MAXLEN option is provided, the command may run in constant time.

**Arity** -3

**ACL categories** LIST

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "element",
    "type": "string"
  },
  {
    "token": "RANK",
    "name": "rank",
    "type": "integer",
    "optional": true
  },
  {
    "token": "COUNT",
    "name": "num-matches",
    "type": "integer",
    "optional": true
  },
  {
    "token": "MAXLEN",
    "name": "len",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "anyOf": [
    {
      "description": "In case there is no matching element",
      "type": "null"
    },
    {
      "description": "An integer representing the matching element",
      "type": "integer"
    },
    {
      "description": "If the COUNT option is given, an array of integers representing the matching elements (empty if there are no matches)",
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "integer"
      }
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
