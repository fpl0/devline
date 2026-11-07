# PING

Returns the server's liveliness response.

## Syntax

```text
PING [message]
```


**Available since** 1.0.0

**Group** connection

**Time complexity** O(1)

**Arity** -1

**ACL categories** CONNECTION

**Command flags** FAST, SENTINEL

## Arguments

```json
[
  {
    "name": "message",
    "type": "string",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "anyOf": [
    {
      "const": "PONG",
      "description": "Default reply."
    },
    {
      "type": "string",
      "description": "Relay of given `message`."
    }
  ]
}
```
