# SLOWLOG GET

Returns the slow log's entries.

## Syntax

```text
SLOWLOG GET [count]
```


**Available since** 2.2.12

**Group** server

**Time complexity** O(N) where N is the number of entries returned

**Arity** -2

**Command flags** ADMIN, LOADING, STALE

## History

- 4.0.0 — Added client IP address, port and name to the reply.
- 8.10.0 — Added original command argument count to the reply.

## Arguments

```json
[
  {
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "Entries from the slow log in chronological order.",
  "uniqueItems": true,
  "items": {
    "type": "array",
    "minItems": 7,
    "maxItems": 7,
    "items": [
      {
        "type": "integer",
        "description": "Slow log entry ID."
      },
      {
        "type": "integer",
        "description": "The unix timestamp at which the logged command was processed.",
        "minimum": 0
      },
      {
        "type": "integer",
        "description": "The amount of time needed for its execution, in microseconds.",
        "minimum": 0
      },
      {
        "type": "array",
        "description": "The arguments of the command.",
        "items": {
          "type": "string"
        }
      },
      {
        "type": "string",
        "description": "Client IP address and port."
      },
      {
        "type": "string",
        "description": "Client name if set via the CLIENT SETNAME command."
      },
      {
        "type": "integer",
        "description": "Total number of arguments in the command."
      }
    ]
  }
}
```
