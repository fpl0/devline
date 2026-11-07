# HELLO

Handshakes with the Redis server.

## Syntax

```text
HELLO [protover [AUTH username password] [SETNAME clientname]]
```


**Available since** 6.0.0

**Group** connection

**Time complexity** O(1)

**Arity** -1

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, FAST, NO_AUTH, SENTINEL, ALLOW_BUSY

## History

- 6.2.0 — `protover` made optional; when called without arguments the command reports the current connection's context.

## Arguments

```json
[
  {
    "name": "arguments",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "protover",
        "type": "integer"
      },
      {
        "token": "AUTH",
        "name": "auth",
        "type": "block",
        "optional": true,
        "arguments": [
          {
            "name": "username",
            "type": "string"
          },
          {
            "name": "password",
            "type": "string"
          }
        ]
      },
      {
        "token": "SETNAME",
        "name": "clientname",
        "type": "string",
        "optional": true
      }
    ]
  }
]
```


## Reply schema

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "server": {
      "type": "string"
    },
    "version": {
      "type": "string"
    },
    "proto": {
      "const": 3
    },
    "id": {
      "type": "integer"
    },
    "mode": {
      "type": "string"
    },
    "role": {
      "type": "string"
    },
    "modules": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "name": {
            "type": "string"
          },
          "ver": {
            "type": "integer"
          },
          "path": {
            "type": "string"
          },
          "args": {
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        }
      }
    }
  }
}
```
