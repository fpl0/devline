# PUNSUBSCRIBE

Stops listening to messages published to channels that match one or more patterns.

## Syntax

```text
PUNSUBSCRIBE [pattern [pattern ...]]
```


**Available since** 2.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of patterns to unsubscribe.

**Arity** -1

**Command flags** PUBSUB, NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "pattern",
    "type": "pattern",
    "optional": true,
    "multiple": true
  }
]
```
