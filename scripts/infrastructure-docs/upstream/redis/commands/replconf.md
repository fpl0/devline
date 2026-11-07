# REPLCONF

An internal command for configuring the replication stream.

## Syntax

```text
REPLCONF (null)
```


**Available since** 3.0.0

**Group** server

**Time complexity** O(1)

**Arity** -1

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, ALLOW_BUSY

**Documentation flags** SYSCMD

## Reply schema

```json
{
  "const": "OK"
}
```
