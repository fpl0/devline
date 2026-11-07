# CLIENT GETREDIR

Returns the client ID to which the connection's tracking notifications are redirected.

## Syntax

```text
CLIENT GETREDIR (null)
```


**Available since** 6.0.0

**Group** connection

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "oneOf": [
    {
      "const": 0,
      "description": "Not redirecting notifications to any client."
    },
    {
      "const": -1,
      "description": "Client tracking is not enabled."
    },
    {
      "type": "integer",
      "description": "ID of the client we are redirecting the notifications to.",
      "minimum": 1
    }
  ]
}
```
