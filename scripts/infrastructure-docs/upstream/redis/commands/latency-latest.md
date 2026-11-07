# LATENCY LATEST

Returns the latest latency samples for all events.

## Syntax

```text
LATENCY LATEST (null)
```


**Available since** 2.8.13

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## Reply schema

```json
{
  "type": "array",
  "description": "An array where each element is a four elements array representing the event's name, timestamp, latest and all-time latency measurements.",
  "items": {
    "type": "array",
    "minItems": 4,
    "maxItems": 4,
    "items": [
      {
        "type": "string",
        "description": "Event name."
      },
      {
        "type": "integer",
        "description": "Timestamp."
      },
      {
        "type": "integer",
        "description": "Latest latency in milliseconds."
      },
      {
        "type": "integer",
        "description": "Max latency in milliseconds."
      }
    ]
  }
}
```
