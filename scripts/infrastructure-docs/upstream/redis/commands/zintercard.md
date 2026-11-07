# ZINTERCARD

Returns the number of members of the intersect of multiple sorted sets.

## Syntax

```text
ZINTERCARD numkeys key [key ...] [LIMIT limit]
```


**Available since** 7.0.0

**Group** sorted_set

**Time complexity** O(N*K) worst case with N being the smallest input sorted set, K being the number of input sorted sets.

**Arity** -3

**ACL categories** SORTEDSET

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
  "description": "Number of elements in the resulting intersection.",
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
