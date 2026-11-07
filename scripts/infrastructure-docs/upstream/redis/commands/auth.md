# AUTH

Authenticates the connection.

## Syntax

```text
AUTH [username] password
```


**Available since** 1.0.0

**Group** connection

**Time complexity** O(N) where N is the number of passwords defined for the user

**Arity** -2

**ACL categories** CONNECTION

**Command flags** NOSCRIPT, LOADING, STALE, FAST, NO_AUTH, SENTINEL, ALLOW_BUSY

## History

- 6.0.0 — Added ACL style (username and password).

## Arguments

```json
[
  {
    "name": "username",
    "type": "string",
    "optional": true,
    "since": "6.0.0"
  },
  {
    "name": "password",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
