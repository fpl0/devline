# ACL WHOAMI

Returns the authenticated username of the current connection.

## Syntax

```text
ACL WHOAMI (null)
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "type": "string",
  "description": "The username of the current connection."
}
```
