# PFCOUNT

Returns the approximated cardinality of the set(s) observed by the HyperLogLog key(s).

## Syntax

```text
PFCOUNT key [key ...]
```


**Available since** 2.8.9

**Group** hyperloglog

**Time complexity** O(1) with a very small average constant time when called with a single key. O(N) with N being the number of keys, and much bigger constant times, when called with multiple keys.

**Arity** -2

**ACL categories** HYPERLOGLOG

**Command flags** READONLY, MAY_REPLICATE

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
  "description": "The approximated number of unique elements observed via PFADD",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "notes": "RW because it may change the internal representation of the key, and propagate to replicas",
    "flags": [
      "RW",
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
