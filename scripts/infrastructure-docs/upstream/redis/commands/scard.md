# SCARD

Returns the number of members in a set.

## Syntax

```text
SCARD key
```


**Available since** 1.0.0

**Group** set

**Time complexity** O(1)

**Arity** 2

**ACL categories** SET

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
  "description": "The cardinality (number of elements) of the set, or 0 if key does not exist.",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO"
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
