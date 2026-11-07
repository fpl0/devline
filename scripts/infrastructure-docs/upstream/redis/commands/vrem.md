# VREM

Remove an element from a vector set

## Syntax

```text
VREM key element
```


**Available since** 8.0.0

**Group** vector_set

**Time complexity** O(log(N)) for each element removed, where N is the number of elements in the vector set.

**Arity** 3

**Command flags** WRITE

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
  }
]
```
