# ZDIFFSTORE

Stores the difference of multiple sorted sets in a key.

## Syntax

```text
ZDIFFSTORE destination numkeys key [key ...]
```


**Available since** 6.2.0

**Group** sorted_set

**Time complexity** O(L + (N-K)log(N)) worst case where L is the total number of elements in all the sets, N is the size of the first set, and K is the size of the result set.

**Arity** -4

**ACL categories** SORTEDSET

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "destination",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 1,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "Number of elements in the resulting sorted set at `destination`",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "OW",
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
  },
  {
    "flags": [
      "RO",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 2
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
