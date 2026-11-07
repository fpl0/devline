# SMOVE

Moves a member from one set to another.

## Syntax

```text
SMOVE source destination member
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(1)

**Arity** 4

**ACL categories** SET

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "source",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "destination",
    "type": "key",
    "key_spec_index": 1
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
      "const": 1,
      "description": "Element is moved."
    },
    {
      "const": 0,
      "description": "The element is not a member of source and no operation was performed."
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
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  },
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
