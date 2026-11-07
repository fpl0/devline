# CLIENT SETINFO

Sets information specific to the client or connection.

## Syntax

```text
CLIENT SETINFO LIB-NAME libname|LIB-VER libver
```


**Available since** 7.2.0

**Group** connection

**Time complexity** O(1)

**Arity** 4

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "attr",
    "type": "oneof",
    "arguments": [
      {
        "token": "lib-name",
        "name": "libname",
        "type": "string"
      },
      {
        "token": "lib-ver",
        "name": "libver",
        "type": "string"
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
