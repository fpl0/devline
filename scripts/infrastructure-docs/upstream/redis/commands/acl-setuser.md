# ACL SETUSER

Creates and modifies an ACL user and its rules.

## Syntax

```text
ACL SETUSER username [rule [rule ...]]
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(N). Where N is the number of rules provided.

**Arity** -3

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## History

- 6.2.0 — Added Pub/Sub channel patterns.
- 7.0.0 — Added selectors and key based permissions.

## Arguments

```json
[
  {
    "name": "username",
    "type": "string"
  },
  {
    "name": "rule",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
