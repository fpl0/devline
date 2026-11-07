# ACL LIST

Dumps the effective rules in ACL file format.

## Syntax

```text
ACL LIST (null)
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
  "description": "A list of currently active ACL rules",
  "items": {
    "type": "string"
  }
}
```
