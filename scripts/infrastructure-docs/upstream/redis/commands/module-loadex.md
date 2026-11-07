# MODULE LOADEX

Loads a module using extended parameters.

## Syntax

```text
MODULE LOADEX path [CONFIG name value [CONFIG name value ...]] [ARGS args [args ...]]
```


**Available since** 7.0.0

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
    "name": "configs",
    "token": "CONFIG",
    "type": "block",
    "multiple": true,
    "multiple_token": true,
    "optional": true,
    "arguments": [
      {
        "name": "name",
        "type": "string"
      },
      {
        "name": "value",
        "type": "string"
      }
    ]
  },
  {
    "name": "args",
    "token": "ARGS",
    "type": "string",
    "multiple": true,
    "optional": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
