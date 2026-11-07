# PFMERGE

Merges one or more HyperLogLog values into a single key.

## Syntax

```text
PFMERGE destkey [sourcekey [sourcekey ...]]
```


**Available since** 2.8.9

**Group** hyperloglog

**Time complexity** O(N) to merge N HyperLogLogs, but with high constant times.

**Arity** -2

**ACL categories** HYPERLOGLOG

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "destkey",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "sourcekey",
    "type": "key",
    "key_spec_index": 1,
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS",
      "INSERT"
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
      "range": {
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
