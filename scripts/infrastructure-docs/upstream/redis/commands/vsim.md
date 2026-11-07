# VSIM

Return elements by vector similarity

## Syntax

```text
VSIM key ELE|FP32|VALUES vector_or_element [WITHSCORES] [WITHATTRIBS] [COUNT count] [EPSILON max_distance] [EF search-exploration-factor] [FILTER expression] [FILTER-EF max-filtering-effort] [TRUTH] [NOTHREAD]
```


**Available since** 8.0.0

**Group** vector_set

**Time complexity** O(log(N)) where N is the number of elements in the vector set.

**Arity** -4

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key"
  },
  {
    "name": "format",
    "type": "oneof",
    "arguments": [
      {
        "name": "ele",
        "type": "pure-token",
        "token": "ELE"
      },
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
    "name": "vector_or_element",
    "type": "string"
  },
  {
    "token": "WITHSCORES",
    "name": "withscores",
    "type": "pure-token",
    "optional": true
  },
  {
    "token": "WITHATTRIBS",
    "name": "withattribs",
    "type": "pure-token",
    "optional": true
  },
  {
    "token": "COUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  },
  {
    "token": "EPSILON",
    "name": "max_distance",
    "type": "double",
    "optional": true
  },
  {
    "token": "EF",
    "name": "search-exploration-factor",
    "type": "integer",
    "optional": true
  },
  {
    "token": "FILTER",
    "name": "expression",
    "type": "string",
    "optional": true
  },
  {
    "token": "FILTER-EF",
    "name": "max-filtering-effort",
    "type": "integer",
    "optional": true
  },
  {
    "token": "TRUTH",
    "name": "truth",
    "type": "pure-token",
    "optional": true
  },
  {
    "token": "NOTHREAD",
    "name": "nothread",
    "type": "pure-token",
    "optional": true
  }
]
```
