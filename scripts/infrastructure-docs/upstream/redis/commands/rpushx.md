# RPUSHX

Appends an element to a list only when the list exists.

## Syntax

```text
RPUSHX key element [element ...]
```


**Available since** 2.2.0

**Group** list

**Time complexity** O(1) for each element added, so O(N) to add N elements when the command is called with multiple arguments.

**Arity** -3

**ACL categories** LIST

**Command flags** WRITE, DENYOOM, FAST

## History

- 4.0.0 — Accepts multiple `element` arguments.

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
  "type": "integer",
  "description": "Length of the list after the push operation.",
  "minimum": 0
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
