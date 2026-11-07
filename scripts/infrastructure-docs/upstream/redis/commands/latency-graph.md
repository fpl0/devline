# LATENCY GRAPH

Returns a latency graph for an event.

## Syntax

```text
LATENCY GRAPH event
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
  "type": "string",
  "description": "Latency graph"
}
```
