# UNSUBSCRIBE

Stops listening to messages posted to channels.

## Syntax

```text
UNSUBSCRIBE [channel [channel ...]]
```


**Available since** 2.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of channels to unsubscribe.

**Arity** -1

**Command flags** PUBSUB, NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "channel",
    "type": "string",
    "optional": true,
    "multiple": true
  }
]
```
