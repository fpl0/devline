# CLIENT CACHING

Instructs the server whether to track the keys in the next request.

## Syntax

```text
CLIENT CACHING YES|NO
```


**Available since** 6.0.0

**Group** connection

**Time complexity** O(1)

**Arity** 3

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "mode",
    "type": "oneof",
    "arguments": [
      {
        "name": "yes",
        "type": "pure-token",
        "token": "YES"
      },
      {
        "name": "no",
        "type": "pure-token",
        "token": "NO"
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
