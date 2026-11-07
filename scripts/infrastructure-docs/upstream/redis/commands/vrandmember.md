# VRANDMEMBER

Return one or multiple random members from a vector set

## Syntax

```text
VRANDMEMBER key [count]
```


**Available since** 8.0.0

**Group** vector_set

**Time complexity** O(N) where N is the absolute value of the count argument.

**Arity** -2

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key"
  },
  {
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```
