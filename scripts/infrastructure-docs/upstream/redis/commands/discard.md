# DISCARD

Discards a transaction.

## Syntax

```text
DISCARD (null)
```


**Available since** 2.0.0

**Group** transactions

**Time complexity** O(N), when N is the number of queued commands

**Arity** 1

**ACL categories** TRANSACTION

**Command flags** NOSCRIPT, LOADING, STALE, FAST, ALLOW_BUSY

## Reply schema

```json
{
  "const": "OK"
}
```
