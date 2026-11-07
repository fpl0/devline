# LATENCY HISTOGRAM

Returns the cumulative distribution of latencies of a subset or all commands.

## Syntax

```text
LATENCY HISTOGRAM [command [command ...]]
```


**Available since** 7.0.0

**Group** server

**Time complexity** O(N) where N is the number of commands with latency information being retrieved.

**Arity** -2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## Arguments

```json
[
  {
    "name": "COMMAND",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "object",
  "description": "A map where each key is a command name, and each value is a map with the total calls, and an inner map of the histogram time buckets.",
  "patternProperties": {
    "^.*$": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "calls": {
          "description": "The total calls for the command.",
          "type": "integer",
          "minimum": 0
        },
        "histogram_usec": {
          "description": "Histogram map, bucket id to latency",
          "type": "object",
          "additionalProperties": {
            "type": "integer"
          }
        }
      }
    }
  }
}
```
