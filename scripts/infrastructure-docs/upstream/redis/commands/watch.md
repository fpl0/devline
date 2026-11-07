# WATCH

Monitors changes to keys to determine the execution of a transaction.

## Syntax

```text
WATCH key [key ...]
```


**Available since** 2.2.0

**Group** transactions

**Time complexity** O(1) for every key.

**Arity** -2

**ACL categories** TRANSACTION

**Command flags** NOSCRIPT, LOADING, STALE, FAST, ALLOW_BUSY

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
  "const": "OK"
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
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
