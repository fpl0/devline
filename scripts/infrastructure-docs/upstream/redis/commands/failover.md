# FAILOVER

Starts a coordinated failover from a server to one of its replicas.

## Syntax

```text
FAILOVER [TO host port [FORCE]] [ABORT] [TIMEOUT milliseconds]
```


**Available since** 6.2.0

**Group** server

**Time complexity** O(1)

**Arity** -1

**Command flags** ADMIN, NOSCRIPT, STALE

## Arguments

```json
[
  {
    "name": "target",
    "token": "TO",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "host",
        "type": "string"
      },
      {
        "name": "port",
        "type": "integer"
      },
      {
        "token": "FORCE",
        "name": "force",
        "type": "pure-token",
        "optional": true
      }
    ]
  },
  {
    "token": "ABORT",
    "name": "abort",
    "type": "pure-token",
    "optional": true
  },
  {
    "token": "TIMEOUT",
    "name": "milliseconds",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
