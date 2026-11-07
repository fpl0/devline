# BGSAVE

Asynchronously saves the database(s) to disk.

## Syntax

```text
BGSAVE [SCHEDULE]
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(1)

**Arity** -1

**Command flags** NO_ASYNC_LOADING, ADMIN, NOSCRIPT

## History

- 3.2.2 — Added the `SCHEDULE` option.

## Arguments

```json
[
  {
    "name": "schedule",
    "token": "SCHEDULE",
    "type": "pure-token",
    "optional": true,
    "since": "3.2.2"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "const": "Background saving started"
    },
    {
      "const": "Background saving scheduled"
    }
  ]
}
```
