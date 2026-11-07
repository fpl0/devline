# CLIENT NO-EVICT

Sets the client eviction mode of the connection.

## Syntax

```text
CLIENT NO-EVICT ON|OFF
```


**Available since** 7.0.0

**Group** connection

**Time complexity** O(1)

**Arity** 3

**ACL categories** CONNECTION

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "enabled",
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
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
