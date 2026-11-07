# PSYNC

An internal command used in replication.

## Syntax

```text
PSYNC replicationid offset
```


**Available since** 2.8.0

**Group** server

**Arity** -3

**Command flags** NO_ASYNC_LOADING, ADMIN, NO_MULTI, NOSCRIPT

## Arguments

```json
[
  {
    "name": "replicationid",
    "type": "string"
  },
  {
    "name": "offset",
    "type": "integer"
  }
]
```
