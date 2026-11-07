# SPUBLISH

Post a message to a shard channel

## Syntax

```text
SPUBLISH shardchannel message
```


**Available since** 7.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of clients subscribed to the receiving shard channel.

**Arity** 3

**Command flags** PUBSUB, LOADING, STALE, FAST, MAY_REPLICATE

## Arguments

```json
[
  {
    "name": "shardchannel",
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


## Key specifications

```json
[
  {
    "flags": [
      "NOT_KEY"
    ],
    "begin_search": {
      "index": {
        "pos": 1
      }
    },
    "find_keys": {
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
