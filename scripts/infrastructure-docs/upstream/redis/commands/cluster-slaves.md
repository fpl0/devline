# CLUSTER SLAVES

Lists the replica nodes of a master node.

## Syntax

```text
CLUSTER SLAVES node-id
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the number of replicas.

**Arity** 3

**Deprecated since** 5.0.0

**Replacement** `CLUSTER REPLICAS`

**Command flags** ADMIN, LOADING, STALE

**Documentation flags** DEPRECATED

## Arguments

```json
[
  {
    "name": "node-id",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "a list of replica nodes replicating from the specified master node provided in the same format used by CLUSTER NODES",
  "type": "array",
  "items": {
    "type": "string",
    "description": "the serialized cluster configuration"
  }
}
```
