# RESET

Resets the connection.

## Syntax

```text
RESET (null)
```


**Available since** 6.2.0

**Group** connection

**Time complexity** O(1)

**Arity** 1

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, FAST, NO_AUTH, ALLOW_BUSY

## Reply schema

```json
{
  "const": "RESET"
}
```
