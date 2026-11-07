# FUNCTION LOAD

Creates a library.

## Syntax

```text
FUNCTION LOAD [REPLACE] function-code
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(1) (considering compilation time is redundant)

**Arity** -3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "replace",
    "type": "pure-token",
    "token": "REPLACE",
    "optional": true
  },
  {
    "name": "function-code",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The library name that was loaded",
  "type": "string"
}
```
