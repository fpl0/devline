# CONFIG SET

Sets configuration parameters in-flight.

## Syntax

```text
CONFIG SET parameter value [parameter value ...]
```


**Available since** 2.0.0

**Group** server

**Time complexity** O(N) when N is the number of configuration parameters provided

**Arity** -4

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE

## History

- 7.0.0 — Added the ability to set multiple parameters in one call.

## Arguments

```json
[
  {
    "name": "data",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "parameter",
        "type": "string"
      },
      {
        "name": "value",
        "type": "string"
      }
    ]
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```
