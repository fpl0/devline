# CLUSTER SETSLOT

Binds a hash slot to a node.

## Syntax

```text
CLUSTER SETSLOT slot IMPORTING node-id|MIGRATING node-id|NODE node-id|STABLE
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** -4

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Arguments

```json
[
  {
    "name": "slot",
    "type": "integer"
  },
  {
    "name": "subcommand",
    "type": "oneof",
    "arguments": [
      {
        "name": "importing",
        "display": "node-id",
        "type": "string",
        "token": "IMPORTING"
      },
      {
        "name": "migrating",
        "display": "node-id",
        "type": "string",
        "token": "MIGRATING"
      },
      {
        "name": "node",
        "display": "node-id",
        "type": "string",
        "token": "NODE"
      },
      {
        "name": "stable",
        "type": "pure-token",
        "token": "STABLE"
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
