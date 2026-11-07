# CLIENT LIST

Lists open connections.

## Syntax

```text
CLIENT LIST [TYPE NORMAL|MASTER|REPLICA|PUBSUB] [ID client-id [client-id ...]]
```


**Available since** 2.4.0

**Group** connection

**Time complexity** O(N) where N is the number of client connections

**Arity** -2

**ACL categories** CONNECTION

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## History

- 2.8.12 — Added unique client `id` field.
- 5.0.0 — Added optional `TYPE` filter.
- 6.0.0 — Added `user` field.
- 6.2.0 — Added `argv-mem`, `tot-mem`, `laddr` and `redir` fields and the optional `ID` filter.
- 7.0.0 — Added `resp`, `multi-mem`, `rbs` and `rbp` fields.
- 7.0.3 — Added `ssub` field.
- 7.2.0 — Added `lib-name` and `lib-ver` fields.
- 7.4.0 — Added `watch` field.
- 8.0.0 — Added `io-thread` field.

## Arguments

```json
[
  {
    "token": "TYPE",
    "name": "client-type",
    "type": "oneof",
    "optional": true,
    "since": "5.0.0",
    "arguments": [
      {
        "name": "normal",
        "type": "pure-token",
        "token": "normal"
      },
      {
        "name": "master",
        "type": "pure-token",
        "token": "master"
      },
      {
        "name": "replica",
        "type": "pure-token",
        "token": "replica"
      },
      {
        "name": "pubsub",
        "type": "pure-token",
        "token": "pubsub"
      }
    ]
  },
  {
    "name": "client-id",
    "token": "ID",
    "type": "integer",
    "optional": true,
    "multiple": true,
    "since": "6.2.0"
  }
]
```


## Reply schema

```json
{
  "type": "string",
  "description": "Information and statistics about client connections"
}
```
