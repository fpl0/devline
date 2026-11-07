# CLIENT UNBLOCK

Unblocks a client blocked by a blocking command from a different connection.

## Syntax

```text
CLIENT UNBLOCK client-id [TIMEOUT|ERROR]
```


**Available since** 5.0.0

**Group** connection

**Time complexity** O(log N) where N is the number of client connections

**Arity** -3

**ACL categories** CONNECTION

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "client-id",
    "type": "integer"
  },
  {
    "name": "unblock-type",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "timeout",
        "type": "pure-token",
        "token": "TIMEOUT"
      },
      {
        "name": "error",
        "type": "pure-token",
        "token": "ERROR"
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
      "const": 0,
      "description": "if the client was unblocked successfully"
    },
    {
      "const": 1,
      "description": "if the client wasn't unblocked"
    }
  ]
}
```
