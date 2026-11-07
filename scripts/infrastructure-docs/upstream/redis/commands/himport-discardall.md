# HIMPORT DISCARDALL

Removes all session-local fieldsets for the connection.

## Syntax

```text
HIMPORT DISCARDALL (null)
```


**Available since** 8.10.0

**Group** hash

**Time complexity** O(N) where N is the number of session-local fieldsets.

**Arity** 2

**ACL categories** HASH

## Reply schema

```json
{
  "type": "integer",
  "description": "Number of fieldsets removed."
}
```
