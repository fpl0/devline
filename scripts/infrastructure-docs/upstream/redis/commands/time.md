# TIME

Returns the server time.

## Syntax

```text
TIME (null)
```


**Available since** 2.6.0

**Group** server

**Time complexity** O(1)

**Arity** 1

**Command flags** LOADING, STALE, FAST

## Reply schema

```json
{
  "type": "array",
  "description": "Array containing two elements: Unix time in seconds and microseconds.",
  "minItems": 2,
  "maxItems": 2,
  "items": {
    "type": "string",
    "pattern": "[0-9]+"
  }
}
```
