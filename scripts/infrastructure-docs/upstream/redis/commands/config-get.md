# CONFIG GET

Returns the effective values of configuration parameters.

## Syntax

```text
CONFIG GET parameter [parameter ...]
```


**Available since** 2.0.0

**Group** server

**Time complexity** O(N) when N is the number of configuration parameters provided

**Arity** -3

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## History

- 7.0.0 — Added the ability to pass multiple pattern parameters in one call

## Arguments

```json
[
  {
    "name": "parameter",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "type": "object",
  "additionalProperties": {
    "type": "string"
  }
}
```
