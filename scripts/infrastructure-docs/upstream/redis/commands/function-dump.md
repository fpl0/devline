# FUNCTION DUMP

Dumps all libraries into a serialized binary payload.

## Syntax

```text
FUNCTION DUMP (null)
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(N) where N is the number of functions

**Arity** 2

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT

## Reply schema

```json
{
  "description": "the serialized payload",
  "type": "string"
}
```
