# MODULE UNLOAD

Unloads a module.

## Syntax

```text
MODULE UNLOAD name
```


**Available since** 4.0.0

**Group** server

**Time complexity** O(1)

**Arity** 3

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT, PROTECTED

## Arguments

```json
[
  {
    "name": "name",
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
