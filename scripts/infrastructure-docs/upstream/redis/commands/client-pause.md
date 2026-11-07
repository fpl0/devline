# CLIENT PAUSE

Suspends commands processing.

## Syntax

```text
CLIENT PAUSE timeout [WRITE|ALL]
```


**Available since** 3.0.0

**Group** connection

**Time complexity** O(1)

**Arity** -3

**ACL categories** CONNECTION

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## History

- 6.2.0 — `CLIENT PAUSE WRITE` mode added along with the `mode` option.

## Arguments

```json
[
  {
    "name": "timeout",
    "type": "integer"
  },
  {
    "name": "mode",
    "type": "oneof",
    "optional": true,
    "since": "6.2.0",
    "arguments": [
      {
        "name": "write",
        "type": "pure-token",
        "token": "WRITE"
      },
      {
        "name": "all",
        "type": "pure-token",
        "token": "ALL"
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
