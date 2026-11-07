# LPUSH

Prepends one or more elements to a list. Creates the key if it doesn't exist.

## Syntax

```text
LPUSH key element [element ...]
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(1) for each element added, so O(N) to add N elements when the command is called with multiple arguments.

**Arity** -3

**ACL categories** LIST

**Command flags** WRITE, DENYOOM, FAST

## History

- 2.4.0 — Accepts multiple `element` arguments.

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
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "Length of the list after the push operations.",
  "type": "integer"
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
