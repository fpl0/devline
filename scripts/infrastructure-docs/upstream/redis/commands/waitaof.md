# WAITAOF

Blocks until all of the preceding write commands sent by the connection are written to the append-only file of the master and/or replicas.

## Syntax

```text
WAITAOF numlocal numreplicas timeout
```


**Available since** 7.2.0

**Group** generic

**Time complexity** O(1)

**Arity** 4

**ACL categories** CONNECTION

**Command flags** BLOCKING

## Arguments

```json
[
  {
    "name": "numlocal",
    "type": "integer"
  },
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
  "type": "array",
  "description": "Number of local and remote AOF files in sync.",
  "minItems": 2,
  "maxItems": 2,
  "items": [
    {
      "description": "Number of local AOF files.",
      "type": "integer",
      "minimum": 0
    },
    {
      "description": "Number of replica AOF files.",
      "type": "number",
      "minimum": 0
    }
  ]
}
```
