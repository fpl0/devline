# CLUSTER INFO

Returns information about the state of a node.

## Syntax

```text
CLUSTER INFO (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(1)

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "description": "A map between named fields and values in the form of <field>:<value> lines separated by newlines composed by the two bytes CRLF",
  "type": "string"
}
```
