# PFADD

Adds elements to a HyperLogLog key. Creates the key if it doesn't exist.

## Syntax

```text
PFADD key [element [element ...]]
```


**Available since** 2.8.9

**Group** hyperloglog

**Time complexity** O(1) to add every element.

**Arity** -2

**ACL categories** HYPERLOGLOG

**Command flags** WRITE, DENYOOM, FAST

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
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "if at least 1 HyperLogLog internal register was altered",
      "const": 1
    },
    {
      "description": "if no HyperLogLog internal register were altered",
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
