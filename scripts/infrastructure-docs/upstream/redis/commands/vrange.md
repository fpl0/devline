# VRANGE

Return elements in a lexicographical range

## Syntax

```text
VRANGE key start end [count]
```


**Available since** 8.4.0

**Group** vector_set

**Time complexity** O(log(K)+M) where K is the number of elements in the start prefix, and M is the number of elements returned. In practical terms, the command is just O(M)

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
    "name": "start",
    "type": "string"
  },
  {
    "name": "end",
    "type": "string"
  },
  {
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```
