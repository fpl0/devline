# CLUSTER MEET

Forces a node to handshake with another node.

## Syntax

```text
CLUSTER MEET ip port [cluster-bus-port]
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** -4

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## History

- 4.0.0 — Added the optional `cluster_bus_port` argument.

## Arguments

```json
[
  {
    "name": "ip",
    "type": "string"
  },
  {
    "name": "port",
    "type": "integer"
  },
  {
    "name": "cluster-bus-port",
    "type": "integer",
    "optional": true,
    "since": "4.0.0"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
