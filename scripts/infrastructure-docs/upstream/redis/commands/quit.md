# QUIT

Closes the connection.

## Syntax

```text
QUIT (null)
```


**Available since** 1.0.0

**Group** connection

**Time complexity** O(1)

**Arity** -1

**Deprecated since** 7.2.0

**Replacement** just closing the connection

**ACL categories** CONNECTION

**Command flags** ALLOW_BUSY, NOSCRIPT, LOADING, STALE, FAST, NO_AUTH

**Documentation flags** DEPRECATED

## Reply schema

```json
{
  "const": "OK"
}
```
