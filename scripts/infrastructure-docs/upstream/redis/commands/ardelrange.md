# ARDELRANGE

Deletes elements in one or more ranges.

## Syntax

```text
ARDELRANGE key start end [start end ...]
```


**Available since** 8.8.0

**Group** array

**Time complexity** Proportional to the number of existing elements / slices touched, not to the numeric span of the requested ranges

**Arity** -4

**ACL categories** ARRAY

**Command flags** WRITE

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "range",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "start",
        "type": "integer"
      },
      {
        "name": "end",
        "type": "integer"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "Number of elements deleted.",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
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
  }
]
```
