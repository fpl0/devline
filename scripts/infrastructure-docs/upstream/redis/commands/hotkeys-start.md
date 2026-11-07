# HOTKEYS START

Starts hotkeys tracking.

## Syntax

```text
HOTKEYS START METRICS count [CPU] [NET] [COUNT k] [DURATION seconds] [SAMPLE ratio] [SLOTS count slot [slot ...]]
```


**Available since** 8.6.0

**Group** server

**Time complexity** O(1)

**Arity** -2

**Command flags** ADMIN, NOSCRIPT

## Arguments

```json
[
  {
    "token": "METRICS",
    "name": "metrics",
    "type": "block",
    "optional": false,
    "arguments": [
      {
        "name": "count",
        "type": "integer"
      },
      {
        "token": "CPU",
        "name": "cpu",
        "type": "pure-token",
        "optional": true
      },
      {
        "token": "NET",
        "name": "net",
        "type": "pure-token",
        "optional": true
      }
    ]
  },
  {
    "token": "COUNT",
    "name": "k",
    "type": "integer",
    "optional": true
  },
  {
    "token": "DURATION",
    "name": "seconds",
    "type": "integer",
    "optional": true
  },
  {
    "token": "SAMPLE",
    "name": "ratio",
    "type": "integer",
    "optional": true
  },
  {
    "token": "SLOTS",
    "name": "slots",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "count",
        "type": "integer"
      },
      {
        "name": "slot",
        "type": "integer",
        "multiple": true
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
