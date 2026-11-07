# VLINKS

Return the neighbors of an element at each layer in the HNSW graph

## Syntax

```text
VLINKS key element [WITHSCORES]
```


**Available since** 8.0.0

**Group** vector_set

**Time complexity** O(1)

**Arity** -3

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key"
  },
  {
    "name": "element",
    "type": "string"
  },
  {
    "token": "WITHSCORES",
    "name": "withscores",
    "type": "pure-token",
    "optional": true
  }
]
```
