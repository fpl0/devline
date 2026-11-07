# BACKUP SEAL

Freeze the current backup (BASE + INCR + manifest).

## Syntax

```text
BACKUP SEAL (null)
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
