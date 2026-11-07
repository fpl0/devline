# MODULE LOAD

Loads a module.

## Syntax

```text
MODULE LOAD path [arg [arg ...]]
```


**Available since** 4.0.0

**Group** server

**Time complexity** O(1)

**Arity** -3

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT, PROTECTED

## Arguments

```json
[
  {
    "name": "path",
    "type": "string"
  },
  {
    "name": "arg",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
