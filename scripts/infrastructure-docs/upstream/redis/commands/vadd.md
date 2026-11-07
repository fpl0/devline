# VADD

Add one or more elements to a vector set, or update its vector if it already exists

## Syntax

```text
VADD key [REDUCE dim] FP32|VALUES vector element [CAS] [NOQUANT|BIN|Q8] [EF build-exploration-factor] [SETATTR attributes] [M numlinks]
```


**Available since** 8.0.0

**Group** vector_set

**Time complexity** O(log(N)) for each element added, where N is the number of elements in the vector set.

**Arity** -5

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "key",
    "type": "key"
  },
  {
    "token": "REDUCE",
    "name": "reduce",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "dim",
        "type": "integer"
      }
    ]
  },
  {
    "name": "format",
    "type": "oneof",
    "arguments": [
      {
        "name": "fp32",
        "type": "pure-token",
        "token": "FP32"
      },
      {
        "name": "values",
        "type": "pure-token",
        "token": "VALUES"
      }
    ]
  },
  {
    "name": "vector",
    "type": "string"
  },
  {
    "name": "element",
    "type": "string"
  },
  {
    "token": "CAS",
    "name": "cas",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "quant_type",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "noquant",
        "type": "pure-token",
        "token": "NOQUANT"
      },
      {
        "name": "bin",
        "type": "pure-token",
        "token": "BIN"
      },
      {
        "name": "q8",
        "type": "pure-token",
        "token": "Q8"
      }
    ]
  },
  {
    "token": "EF",
    "name": "build-exploration-factor",
    "type": "integer",
    "optional": true
  },
  {
    "token": "SETATTR",
    "name": "attributes",
    "type": "string",
    "optional": true
  },
  {
    "token": "M",
    "name": "numlinks",
    "type": "integer",
    "optional": true
  }
]
```
