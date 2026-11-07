# ACL LOG

Lists recent security events generated due to ACL rules.

## Syntax

```text
ACL LOG [count|RESET]
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(N) with N being the number of entries shown.

**Arity** -2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, SENTINEL

## History

- 7.2.0 — Added entry ID, timestamp created, and timestamp last updated.

## Arguments

```json
[
  {
    "name": "operation",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "count",
        "type": "integer"
      },
      {
        "name": "reset",
        "type": "pure-token",
        "token": "RESET"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "In case `RESET` was not given, a list of recent ACL security events.",
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "count": {
            "type": "integer"
          },
          "reason": {
            "type": "string"
          },
          "context": {
            "type": "string"
          },
          "object": {
            "type": "string"
          },
          "username": {
            "type": "string"
          },
          "age-seconds": {
            "type": "number"
          },
          "client-info": {
            "type": "string"
          },
          "entry-id": {
            "type": "integer"
          },
          "timestamp-created": {
            "type": "integer"
          },
          "timestamp-last-updated": {
            "type": "integer"
          }
        }
      }
    },
    {
      "const": "OK",
      "description": "In case `RESET` was given, OK indicates ACL log was cleared."
    }
  ]
}
```
