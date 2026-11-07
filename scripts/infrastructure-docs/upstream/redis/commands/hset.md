# HSET

Creates or modifies the value of a field in a hash.

## Syntax

```text
HSET key field value [field value ...]
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(1) for each field/value pair added, so O(N) to add N field/value pairs when the command is called with multiple field/value pairs.

**Arity** -4

**ACL categories** HASH

**Command flags** WRITE, DENYOOM, FAST

## History

- 4.0.0 — Accepts multiple `field` and `value` arguments.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "data",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "field",
        "type": "string"
      },
      {
        "name": "value",
        "type": "string"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "description": "The number of fields that were added",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "UPDATE"
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
