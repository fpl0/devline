# ARLASTITEMS

Returns the most recently inserted elements.

## Syntax

```text
ARLASTITEMS key count [REV]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(N) where N is the count

**Arity** -3

**ACL categories** ARRAY

**Command flags** READONLY

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
    "type": "integer"
  },
  {
    "name": "rev",
    "type": "pure-token",
    "token": "REV",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "items": {
    "oneOf": [
      {
        "type": "string"
      },
      {
        "type": "null"
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
