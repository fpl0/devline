# FUNCTION FLUSH

Deletes all libraries and functions.

## Syntax

```text
FUNCTION FLUSH [ASYNC|SYNC]
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(N) where N is the number of functions deleted

**Arity** -2

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, WRITE

## Arguments

```json
[
  {
    "name": "flush-type",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "async",
        "type": "pure-token",
        "token": "ASYNC"
      },
      {
        "name": "sync",
        "type": "pure-token",
        "token": "SYNC"
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
