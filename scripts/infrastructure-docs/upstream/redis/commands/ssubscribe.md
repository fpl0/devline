# SSUBSCRIBE

Listens for messages published to shard channels.

## Syntax

```text
SSUBSCRIBE shardchannel [shardchannel ...]
```


**Available since** 7.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of shard channels to subscribe to.

**Arity** -2

**Command flags** PUBSUB, NOSCRIPT, LOADING, STALE, DENYOOM

## Arguments

```json
[
  {
    "name": "shardchannel",
    "type": "string",
    "multiple": true
  }
]
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
        "lastkey": -1,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
