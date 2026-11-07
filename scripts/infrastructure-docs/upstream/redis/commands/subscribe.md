# SUBSCRIBE

Listens for messages published to channels.

## Syntax

```text
SUBSCRIBE channel [channel ...]
```


**Available since** 2.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of channels to subscribe to.

**Arity** -2

**Command flags** PUBSUB, NOSCRIPT, LOADING, STALE, SENTINEL, DENYOOM

## Arguments

```json
[
  {
    "name": "channel",
    "type": "string",
    "multiple": true
  }
]
```
