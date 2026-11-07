# SCRIPT DEBUG

Sets the debug mode of server-side Lua scripts.

## Syntax

```text
SCRIPT DEBUG YES|SYNC|NO
```


**Available since** 3.2.0

**Group** scripting

**Time complexity** O(1)

**Arity** 3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT

## Arguments

```json
[
  {
    "name": "mode",
    "type": "oneof",
    "arguments": [
      {
        "name": "yes",
        "type": "pure-token",
        "token": "YES"
      },
      {
        "name": "sync",
        "type": "pure-token",
        "token": "SYNC"
      },
      {
        "name": "no",
        "type": "pure-token",
        "token": "NO"
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
