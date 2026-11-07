# VSETATTR

Associate or remove the JSON attributes of elements

## Syntax

```text
VSETATTR key element json
```


**Available since** 8.0.0

**Group** vector_set

**Time complexity** O(1)

**Arity** 4

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
  },
  {
    "name": "json",
    "type": "string"
  }
]
```
