# EXEC

Executes all commands in a transaction.

## Syntax

```text
EXEC (null)
```


**Available since** 1.2.0

**Group** transactions

**Time complexity** Depends on commands in the transaction

**Arity** 1

**ACL categories** TRANSACTION

**Command flags** NOSCRIPT, LOADING, STALE, SKIP_SLOWLOG

## Reply schema

```json
{
  "oneOf": [
    {
      "description": "Each element being the reply to each of the commands in the atomic transaction.",
      "type": "array"
    },
    {
      "description": "The transaction was aborted because a `WATCH`ed key was touched",
      "type": "null"
    }
  ]
}
```
