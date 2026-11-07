# FUNCTION RESTORE

Restores all libraries from a payload.

## Syntax

```text
FUNCTION RESTORE serialized-value [FLUSH|APPEND|REPLACE]
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(N) where N is the number of functions on the payload

**Arity** -3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "serialized-value",
    "type": "string"
  },
  {
    "name": "policy",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "flush",
        "type": "pure-token",
        "token": "FLUSH"
      },
      {
        "name": "append",
        "type": "pure-token",
        "token": "APPEND"
      },
      {
        "name": "replace",
        "type": "pure-token",
        "token": "REPLACE"
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
