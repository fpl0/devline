# VEMB

Return the vector associated with an element

## Syntax

```text
VEMB key element [RAW]
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
    "token": "RAW",
    "name": "raw",
    "type": "pure-token",
    "optional": true
  }
]
```
