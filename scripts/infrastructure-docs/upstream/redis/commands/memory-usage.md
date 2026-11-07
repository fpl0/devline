# MEMORY USAGE

Estimates the memory usage of a key.

## Syntax

```text
MEMORY USAGE key [SAMPLES count]
```


**Available since** 4.0.0

**Group** server

**Time complexity** O(N) where N is the number of samples.

**Arity** -3

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
    "token": "SAMPLES",
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "Number of bytes that a key and its value require to be stored in RAM.",
      "type": "integer"
    },
    {
      "description": "Key does not exist.",
      "type": "null"
    }
  ]
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
