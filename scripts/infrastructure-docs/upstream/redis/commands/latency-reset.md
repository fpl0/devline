# LATENCY RESET

Resets the latency data for one or more events.

## Syntax

```text
LATENCY RESET [event [event ...]]
```


**Available since** 2.8.13

**Group** server

**Time complexity** O(1)

**Arity** -2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## Arguments

```json
[
  {
    "name": "event",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "Number of event time series that were reset."
}
```
