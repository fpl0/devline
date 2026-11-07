# CLIENT TRACKING

Controls server-assisted client-side caching for the connection.

## Syntax

```text
CLIENT TRACKING ON|OFF [REDIRECT client-id] [PREFIX prefix [PREFIX prefix ...]] [BCAST] [OPTIN] [OPTOUT] [NOLOOP]
```


**Available since** 6.0.0

**Group** connection

**Time complexity** O(1). Some options may introduce additional complexity.

**Arity** -3

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "status",
    "type": "oneof",
    "arguments": [
      {
        "name": "on",
        "type": "pure-token",
        "token": "ON"
      },
      {
        "name": "off",
        "type": "pure-token",
        "token": "OFF"
      }
    ]
  },
  {
    "token": "REDIRECT",
    "name": "client-id",
    "type": "integer",
    "optional": true
  },
  {
    "token": "PREFIX",
    "name": "prefix",
    "type": "string",
    "optional": true,
    "multiple": true,
    "multiple_token": true
  },
  {
    "name": "BCAST",
    "token": "BCAST",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "OPTIN",
    "token": "OPTIN",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "OPTOUT",
    "token": "OPTOUT",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "NOLOOP",
    "token": "NOLOOP",
    "type": "pure-token",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "if the client was successfully put into or taken out of tracking mode",
  "const": "OK"
}
```
