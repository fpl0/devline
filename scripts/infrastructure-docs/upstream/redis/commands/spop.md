# SPOP

Returns one or more random members from a set after removing them. Deletes the set if the last member was popped.

## Syntax

```text
SPOP key [count]
```


**Available since** 1.0.0

**Group** set

**Time complexity** Without the count argument O(1), otherwise O(N) where N is the value of the passed count.

**Arity** -2

**ACL categories** SET

**Command flags** WRITE, FAST

## History

- 3.2.0 — Added the `count` argument.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "count",
    "type": "integer",
    "optional": true,
    "since": "3.2.0"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "type": "null",
      "description": "The key does not exist."
    },
    {
      "type": "string",
      "description": "The removed member when 'COUNT' is not given."
    },
    {
      "type": "array",
      "description": "List to the removed members when 'COUNT' is given.",
      "uniqueItems": true,
      "items": {
        "type": "string"
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
  }
]
```
