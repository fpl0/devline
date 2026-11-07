# LATENCY DOCTOR

Returns a human-readable latency analysis report.

## Syntax

```text
LATENCY DOCTOR (null)
```


**Available since** 2.8.13

**Group** server

**Time complexity** O(1)

**Arity** 2

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## Reply schema

```json
{
  "type": "string",
  "description": "A human readable latency analysis report."
}
```
