# CLIENT KILL

Terminates open connections.

## Syntax

```text
CLIENT KILL ip:port|[ID client-id]|[TYPE NORMAL|MASTER|SLAVE|REPLICA|PUBSUB]|[USER username]|[ADDR ip:port]|[LADDR ip:port]|[SKIPME YES|NO]|[MAXAGE maxage] [[ID client-id]|[TYPE NORMAL|MASTER|SLAVE|REPLICA|PUBSUB]|[USER username]|[ADDR ip:port]|[LADDR ip:port]|[SKIPME YES|NO]|[MAXAGE maxage] ...]
```


**Available since** 2.4.0

**Group** connection

**Time complexity** O(N) where N is the number of client connections

**Arity** -3

**ACL categories** CONNECTION

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## History

- 2.8.12 — Added new filter format.
- 2.8.12 — `ID` option.
- 3.2.0 — Added `master` type in for `TYPE` option.
- 5.0.0 — Replaced `slave` `TYPE` with `replica`. `slave` still supported for backward compatibility.
- 6.2.0 — `LADDR` option.
- 7.4.0 — `MAXAGE` option.

## Arguments

```json
[
  {
    "name": "filter",
    "type": "oneof",
    "arguments": [
      {
        "name": "old-format",
        "display": "ip:port",
        "type": "string",
        "deprecated_since": "2.8.12"
      },
      {
        "name": "new-format",
        "type": "oneof",
        "multiple": true,
        "arguments": [
          {
            "token": "ID",
            "name": "client-id",
            "type": "integer",
            "optional": true,
            "since": "2.8.12"
          },
          {
            "token": "TYPE",
            "name": "client-type",
            "type": "oneof",
            "optional": true,
            "since": "2.8.12",
            "arguments": [
              {
                "name": "normal",
                "type": "pure-token",
                "token": "normal"
              },
              {
                "name": "master",
                "type": "pure-token",
                "token": "master",
                "since": "3.2.0"
              },
              {
                "name": "slave",
                "type": "pure-token",
                "token": "slave"
              },
              {
                "name": "replica",
                "type": "pure-token",
                "token": "replica",
                "since": "5.0.0"
              },
              {
                "name": "pubsub",
                "type": "pure-token",
                "token": "pubsub"
              }
            ]
          },
          {
            "token": "USER",
            "name": "username",
            "type": "string",
            "optional": true
          },
          {
            "token": "ADDR",
            "name": "addr",
            "display": "ip:port",
            "type": "string",
            "optional": true
          },
          {
            "token": "LADDR",
            "name": "laddr",
            "display": "ip:port",
            "type": "string",
            "optional": true,
            "since": "6.2.0"
          },
          {
            "token": "SKIPME",
            "name": "skipme",
            "type": "oneof",
            "optional": true,
            "arguments": [
              {
                "name": "yes",
                "type": "pure-token",
                "token": "YES"
              },
              {
                "name": "no",
                "type": "pure-token",
                "token": "NO"
              }
            ]
          },
          {
            "token": "MAXAGE",
            "name": "maxage",
            "type": "integer",
            "optional": true,
            "since": "7.4.0"
          }
        ]
      }
    ]
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "when called in 3 argument format",
      "const": "OK"
    },
    {
      "description": "when called in filter/value format, the number of clients killed",
      "type": "integer",
      "minimum": 0
    }
  ]
}
```
