# FLUSHDB

Remove all keys from the current database.

## Syntax

```text
FLUSHDB [ASYNC|SYNC]
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(N) where N is the number of keys in the selected database

**Arity** -1

**ACL categories** KEYSPACE, DANGEROUS

**Command flags** WRITE

## History

- 4.0.0 — Added the `ASYNC` flushing mode modifier.
- 6.2.0 — Added the `SYNC` flushing mode modifier.

## Arguments

```json
[
  {
    "name": "flush-type",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "async",
        "type": "pure-token",
        "token": "ASYNC",
        "since": "4.0.0"
      },
      {
        "name": "sync",
        "type": "pure-token",
        "token": "SYNC",
        "since": "6.2.0"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
