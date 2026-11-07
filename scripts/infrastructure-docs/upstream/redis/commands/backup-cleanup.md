# BACKUP CLEANUP

Remove a sealed backup's files and return to idle.

## Syntax

```text
BACKUP CLEANUP (null)
```


**Available since** 8.10.0

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** ADMIN, NOSCRIPT

## Reply schema

```json
{
  "const": "OK"
}
```
