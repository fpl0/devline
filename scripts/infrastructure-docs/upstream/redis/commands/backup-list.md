# BACKUP LIST

List the immutable backup file paths pinned so far.

## Syntax

```text
BACKUP LIST (null)
```


**Available since** 8.10.0

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** ADMIN, STALE

## Reply schema

```json
{
  "type": "array",
  "description": "The absolute paths of immutable files pinned so far.",
  "items": {
    "type": "string"
  }
}
```
