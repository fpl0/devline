# REPLICAOF

Configures a server as replica of another, or promotes it to a master.

## Syntax

```text
REPLICAOF host port|NO ONE
```


**Available since** 5.0.0

**Group** server

**Time complexity** O(1)

**Arity** 3

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT, STALE

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
  "description": "replicaOf status",
  "type": "string",
  "pattern": "OK*"
}
```
