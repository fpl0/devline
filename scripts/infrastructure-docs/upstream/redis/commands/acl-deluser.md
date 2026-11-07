# ACL DELUSER

Deletes ACL users, and terminates their connections.

## Syntax

```text
ACL DELUSER username [username ...]
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(1) amortized time considering the typical user.

**Arity** -3

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "username",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "The number of users that were deleted"
}
```
