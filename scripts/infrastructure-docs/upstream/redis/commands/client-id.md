# CLIENT ID

Returns the unique client ID of the connection.

## Syntax

```text
CLIENT ID (null)
```


**Available since** 5.0.0

**Group** connection

**Time complexity** O(1)

**Arity** 2

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "type": "integer",
  "description": "The id of the client"
}
```
