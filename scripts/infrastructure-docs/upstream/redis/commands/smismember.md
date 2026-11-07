# SMISMEMBER

Determines whether multiple members belong to a set.

## Syntax

```text
SMISMEMBER key member [member ...]
```


**Available since** 6.2.0

**Group** set

**Time complexity** O(N) where N is the number of elements being checked for membership

**Arity** -3

**ACL categories** SET

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
  "description": "List representing the membership of the given elements, in the same order as they are requested.",
  "minItems": 1,
  "items": {
    "oneOf": [
      {
        "const": 0,
        "description": "Not a member of the set or the key does not exist."
      },
      {
        "const": 1,
        "description": "A member of the set."
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
