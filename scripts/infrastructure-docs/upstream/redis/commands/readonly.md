# READONLY

Enables read-only queries for a connection to a Redis Cluster replica node.

## Syntax

```text
READONLY (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 1

**ACL categories** CONNECTION

**Command flags** FAST, LOADING, STALE

## Reply schema

```json
{
  "const": "OK"
}
```
