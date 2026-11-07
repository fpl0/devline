# SHUTDOWN

Synchronously saves the database(s) to disk and shuts down the Redis server.

## Syntax

```text
SHUTDOWN [NOSAVE|SAVE] [NOW] [FORCE] [ABORT]
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(N) when saving, where N is the total number of keys in all databases when saving data, otherwise O(1)

**Arity** -1

**Command flags** ADMIN, NOSCRIPT, LOADING, STALE, NO_MULTI, SENTINEL, ALLOW_BUSY

## History

- 7.0.0 — Added the `NOW`, `FORCE` and `ABORT` modifiers.

## Arguments

```json
[
  {
    "name": "save-selector",
    "type": "oneof",
    "optional": true,
    "arguments": [
      {
        "name": "nosave",
        "type": "pure-token",
        "token": "NOSAVE"
      },
      {
        "name": "save",
        "type": "pure-token",
        "token": "SAVE"
      }
    ]
  },
  {
    "name": "now",
    "type": "pure-token",
    "token": "NOW",
    "optional": true,
    "since": "7.0.0"
  },
  {
    "name": "force",
    "type": "pure-token",
    "token": "FORCE",
    "optional": true,
    "since": "7.0.0"
  },
  {
    "name": "abort",
    "type": "pure-token",
    "token": "ABORT",
    "optional": true,
    "since": "7.0.0"
  }
]
```


## Reply schema

```json
{
  "description": "OK if ABORT was specified and shutdown was aborted. On successful shutdown, nothing is returned since the server quits and the connection is closed. On failure, an error is returned.",
  "const": "OK"
}
```
