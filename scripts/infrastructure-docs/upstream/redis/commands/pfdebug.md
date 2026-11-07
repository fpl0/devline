# PFDEBUG

Internal commands for debugging HyperLogLog values.

## Syntax

```text
PFDEBUG subcommand key
```


**Available since** 2.8.9

**Group** hyperloglog

**Time complexity** N/A

**Arity** 3

**ACL categories** HYPERLOGLOG

**Command flags** WRITE, DENYOOM, ADMIN

**Documentation flags** SYSCMD

## Arguments

```json
[
  {
    "name": "subcommand",
    "type": "string"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  }
]
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 2
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
