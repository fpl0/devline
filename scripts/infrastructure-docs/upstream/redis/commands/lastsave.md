# LASTSAVE

Returns the Unix timestamp of the last successful save to disk.

## Syntax

```text
LASTSAVE (null)
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(1)

**Arity** 1

**ACL categories** ADMIN, DANGEROUS

**Command flags** LOADING, STALE, FAST

## Reply schema

```json
{
  "type": "integer",
  "description": "UNIX TIME of the last DB save executed with success."
}
```
