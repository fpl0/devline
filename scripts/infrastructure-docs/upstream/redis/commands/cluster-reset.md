# CLUSTER RESET

Resets a node.

## Syntax

```text
CLUSTER RESET [HARD|SOFT]
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the number of known nodes. The command may execute a FLUSHALL as a side effect.

**Arity** -2

**Command flags** ADMIN, STALE, NOSCRIPT

## Arguments

```json
[
  {
    "name": "reset-type",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "hard",
        "type": "pure-token",
        "token": "HARD"
      },
      {
        "name": "soft",
        "type": "pure-token",
        "token": "SOFT"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
