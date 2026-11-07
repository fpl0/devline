# UNWATCH

Forgets about watched keys of a transaction.

## Syntax

```text
UNWATCH (null)
```


**Available since** 2.2.0

**Group** transactions

**Time complexity** O(1)

**Arity** 1

**ACL categories** TRANSACTION

**Command flags** NOSCRIPT, LOADING, STALE, FAST, ALLOW_BUSY

## Reply schema

```json
{
  "const": "OK"
}
```
