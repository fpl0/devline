# ARNEXT

Returns the next index ARINSERT would use.

## Syntax

```text
ARNEXT key
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(1)

**Arity** 2

**ACL categories** ARRAY

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The next index ARINSERT would use. Returns 0 for missing keys or when no insert happened yet.",
      "type": "integer"
    },
    {
      "description": "Null when the insertion cursor is exhausted (next insert would overflow).",
      "type": "null"
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
