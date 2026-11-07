# CLIENT REPLY

Instructs the server whether to reply to commands.

## Syntax

```text
CLIENT REPLY ON|OFF|SKIP
```


**Available since** 3.2.0

**Group** connection

**Time complexity** O(1)

**Arity** 3

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "action",
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
      },
      {
        "name": "skip",
        "type": "pure-token",
        "token": "SKIP"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK",
  "description": "When called with either OFF or SKIP subcommands, no reply is made. When called with ON, reply is OK."
}
```
