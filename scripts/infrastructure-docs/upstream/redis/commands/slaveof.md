# SLAVEOF

Sets a Redis server as a replica of another, or promotes it to being a master.

## Syntax

```text
SLAVEOF host port|NO ONE
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(1)

**Arity** 3

**Deprecated since** 5.0.0

**Replacement** `REPLICAOF`

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT, STALE

**Documentation flags** DEPRECATED

## Arguments

```json
[
  {
    "name": "args",
    "type": "oneof",
    "arguments": [
      {
        "name": "host-port",
        "type": "block",
        "arguments": [
          {
            "name": "host",
            "type": "string"
          },
          {
            "name": "port",
            "type": "integer"
          }
        ]
      },
      {
        "name": "no-one",
        "type": "block",
        "arguments": [
          {
            "name": "no",
            "type": "pure-token",
            "token": "NO"
          },
          {
            "name": "one",
            "type": "pure-token",
            "token": "ONE"
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
  "description": "slaveOf status",
  "type": "string",
  "pattern": "OK*"
}
```
