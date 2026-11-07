# FUNCTION STATS

Returns information about a function during execution.

## Syntax

```text
FUNCTION STATS (null)
```


**Available since** 7.0.0

**Group** scripting

**Time complexity** O(1)

**Arity** 2

**ACL categories** SCRIPTING

**Command flags** NOSCRIPT, ALLOW_BUSY

## Reply schema

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "running_script": {
      "description": "information about the running script.",
      "oneOf": [
        {
          "description": "If there's no in-flight function",
          "type": "null"
        },
        {
          "description": "a map with the information about the running script",
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "name": {
              "description": "the name of the function.",
              "type": "string"
            },
            "command": {
              "description": "the command and arguments used for invoking the function.",
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "duration_ms": {
              "description": "the function's runtime duration in milliseconds.",
              "type": "integer"
            }
          }
        }
      ]
    },
    "engines": {
      "description": "A map when each entry in the map represent a single engine.",
      "type": "object",
      "patternProperties": {
        "^.*$": {
          "description": "Engine map contains statistics about the engine",
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "libraries_count": {
              "description": "number of libraries",
              "type": "integer"
            },
            "functions_count": {
              "description": "number of functions",
              "type": "integer"
            }
          }
        }
      }
    }
  }
}
```
