# LATENCY HISTORY

Returns timestamp-latency samples for an event.

## Syntax

```text
LATENCY HISTORY event
```


**Available since** 2.8.13

**Group** server

**Time complexity** O(1)

**Arity** 3

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## Arguments

```json
[
  {
    "name": "event",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "An array where each element is a two elements array representing the timestamp and the latency of the event.",
  "items": {
    "type": "array",
    "minItems": 2,
    "maxItems": 2,
    "items": [
      {
        "description": "timestamp of the event",
        "type": "integer",
        "minimum": 0
      },
      {
        "description": "latency of the event",
        "type": "integer",
        "minimum": 0
      }
    ]
  }
}
```
