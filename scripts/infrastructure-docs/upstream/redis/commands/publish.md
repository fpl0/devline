# PUBLISH

Posts a message to a channel.

## Syntax

```text
PUBLISH channel message
```


**Available since** 2.0.0

**Group** pubsub

**Time complexity** O(N+M) where N is the number of clients subscribed to the receiving channel and M is the total number of subscribed patterns (by any client).

**Arity** 3

**Command flags** PUBSUB, LOADING, STALE, FAST, MAY_REPLICATE, SENTINEL

## Arguments

```json
[
  {
    "name": "channel",
    "type": "string"
  },
  {
    "name": "message",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "the number of clients that received the message. Note that in a Redis Cluster, only clients that are connected to the same node as the publishing client are included in the count",
  "type": "integer",
  "minimum": 0
}
```
