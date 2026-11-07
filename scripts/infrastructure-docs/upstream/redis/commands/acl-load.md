# ACL LOAD

Reloads the rules from the configured ACL file.

## Syntax

```text
ACL LOAD (null)
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(N). Where N is the number of configured users.

**Arity** 2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "const": "OK"
}
```
