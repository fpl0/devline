# CLIENT INFO

Returns information about the connection.

## Syntax

```text
CLIENT INFO (null)
```


**Available since** 6.2.0

**Group** connection

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "description": "a unique string, as described at the CLIENT LIST page, for the current client",
  "type": "string"
}
```
