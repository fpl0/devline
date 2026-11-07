# BACKUP STATUS

Report the current backup state.

## Syntax

```text
BACKUP STATUS (null)
```


**Available since** 8.10.0

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** ADMIN, STALE

## Reply schema

```json
{
  "type": "object",
  "description": "Current backup state.",
  "additionalProperties": false,
  "properties": {
    "state": {
      "oneOf": [
        {
          "const": "idle"
        },
        {
          "const": "pending"
        },
        {
          "const": "snapshotting"
        },
        {
          "const": "incrementing"
        },
        {
          "const": "sealed"
        },
        {
          "const": "failed"
        }
      ]
    },
    "error": {
      "type": "string"
    },
    "start_time": {
      "type": "integer"
    },
    "end_time": {
      "type": "integer"
    }
  }
}
```
