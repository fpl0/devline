# SINTER

Returns the intersect of multiple sets.

## Syntax

```text
SINTER key [key ...]
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(N*M) worst case where N is the cardinality of the smallest set and M is the number of sets.

**Arity** -2

**ACL categories** SET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "List with the members of the resulting set.",
  "uniqueItems": true,
  "items": {
    "type": "string"
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
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
