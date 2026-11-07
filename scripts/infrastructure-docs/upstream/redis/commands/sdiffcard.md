# SDIFFCARD

Returns the number of members of the difference between the first set and all successive sets.

## Syntax

```text
SDIFFCARD numkeys key [key ...] [LIMIT limit]
```


**Available since** 8.10.0

**Group** set

**Time complexity** O(N) where N is the total number of elements in all given sets.

**Arity** -3

**ACL categories** SET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  },
  {
    "token": "LIMIT",
    "name": "limit",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "Number of the elements in the resulting difference.",
  "type": "integer",
  "minimum": 0
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
      "keynum": {
        "keynumidx": 0,
        "firstkey": 1,
        "step": 1
      }
    }
  }
]
```
