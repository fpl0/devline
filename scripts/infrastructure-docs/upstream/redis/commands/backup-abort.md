# BACKUP ABORT

Cancel a backup that has not been sealed yet.

## Syntax

```text
BACKUP ABORT (null)
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
