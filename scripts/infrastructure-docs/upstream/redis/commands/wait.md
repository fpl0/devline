# WAIT

Blocks until the asynchronous replication of all preceding write commands sent by the connection is completed.

## Syntax

```text
WAIT numreplicas timeout
```


**Available since** 3.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 3

**ACL categories** CONNECTION

**Command flags** BLOCKING

## Arguments

```json
[
  {
    "name": "numreplicas",
    "type": "integer"
  },
  {
    "name": "timeout",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "The number of replicas reached by all the writes performed in the context of the current connection.",
  "minimum": 0
}
```
