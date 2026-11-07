# CLIENT UNPAUSE

Resumes processing commands from paused clients.

## Syntax

```text
CLIENT UNPAUSE (null)
```


**Available since** 6.2.0

**Group** connection

**Time complexity** O(N) Where N is the number of paused clients

**Arity** 2

**ACL categories** CONNECTION

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Reply schema

```json
{
  "const": "OK"
}
```
