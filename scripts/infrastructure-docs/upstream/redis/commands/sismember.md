# SISMEMBER

Determines whether a member belongs to a set.

## Syntax

```text
SISMEMBER key member
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(1)

**Arity** 3

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
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "const": 0,
      "description": "The element is not a member of the set, or the key does not exist."
    },
    {
      "const": 1,
      "description": "The element is a member of the set."
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO"
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
