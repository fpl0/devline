# MODULE LIST

Returns all loaded modules.

## Syntax

```text
MODULE LIST (null)
```


**Available since** 4.0.0

**Group** server

**Time complexity** O(N) where N is the number of loaded modules.

**Arity** 2

**Command flags** ADMIN, NOSCRIPT

## Reply schema

```json
{
  "type": "array",
  "description": "Returns information about the modules loaded to the server.",
  "items": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "name": {
        "type": "string",
        "description": "Name of the module."
      },
      "ver": {
        "type": "integer",
        "description": "Version of the module."
      },
      "path": {
        "type": "string",
        "description": "Module path."
      },
      "args": {
        "type": "array",
        "description": "Module arguments.",
        "items": {
          "type": "string"
        }
      }
    }
  }
}
```
