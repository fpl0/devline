# ECHO

Returns the given string.

## Syntax

```text
ECHO message
```


**Available since** 1.0.0

**Group** connection

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** LOADING, STALE, FAST

## Arguments

```json
[
  {
    "name": "message",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The given string",
  "type": "string"
}
```
