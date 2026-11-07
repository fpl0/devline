# FUNCTION DELETE

Deletes a library and its functions.

## Syntax

```text
FUNCTION DELETE library-name
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(1)

**Arity** 3

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, WRITE

## Arguments

```json
[
  {
    "name": "library-name",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
