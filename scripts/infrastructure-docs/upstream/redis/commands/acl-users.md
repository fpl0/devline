# ACL USERS

Lists all ACL users.

## Syntax

```text
ACL USERS (null)
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(N). Where N is the number of configured users.

**Arity** 2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "type": "array",
  "description": "List of existing ACL users",
  "items": {
    "type": "string"
  }
}
```
