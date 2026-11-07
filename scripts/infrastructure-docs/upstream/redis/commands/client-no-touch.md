# CLIENT NO-TOUCH

Controls whether commands sent by the client affect the LRU/LFU of accessed keys.

## Syntax

```text
CLIENT NO-TOUCH ON|OFF
```


**Available since** 7.2.0

**Group** connection

**Time complexity** O(1)

**Arity** 3

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE

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
