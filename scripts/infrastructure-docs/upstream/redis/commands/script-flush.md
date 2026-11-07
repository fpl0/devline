# SCRIPT FLUSH

Removes all server-side Lua scripts from the script cache.

## Syntax

```text
SCRIPT FLUSH [ASYNC|SYNC]
```


**Available since** 2.6.0

**Group** scripting

**Time complexity** O(N) with N being the number of scripts in cache

**Arity** -2

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT

## History

- 6.2.0 — Added the `ASYNC` and `SYNC` flushing mode modifiers.

## Arguments

```json
[
  {
    "name": "flush-type",
    "type": "oneof",
    "optional": true,
    "since": "6.2.0",
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
