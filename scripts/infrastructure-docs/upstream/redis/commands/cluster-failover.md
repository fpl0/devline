# CLUSTER FAILOVER

Forces a replica to perform a manual failover of its master.

## Syntax

```text
CLUSTER FAILOVER [FORCE|TAKEOVER]
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** -2

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Arguments

```json
[
  {
    "name": "options",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "force",
        "type": "pure-token",
        "token": "FORCE"
      },
      {
        "name": "takeover",
        "type": "pure-token",
        "token": "TAKEOVER"
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
