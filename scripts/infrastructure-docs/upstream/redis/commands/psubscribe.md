# PSUBSCRIBE

Listens for messages published to channels that match one or more patterns.

## Syntax

```text
PSUBSCRIBE pattern [pattern ...]
```


**Available since** 2.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of patterns to subscribe to.

**Arity** -2

**Command flags** PUBSUB, NOSCRIPT, LOADING, STALE, SENTINEL, DENYOOM

## Arguments

```json
[
  {
    "name": "pattern",
    "type": "pattern",
    "multiple": true
  }
]
```
