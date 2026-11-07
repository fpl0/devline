# SUNSUBSCRIBE

Stops listening to messages posted to shard channels.

## Syntax

```text
SUNSUBSCRIBE [shardchannel [shardchannel ...]]
```


**Available since** 7.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of shard channels to unsubscribe.

**Arity** -1

**Command flags** PUBSUB, NOSCRIPT, LOADING, STALE

## Arguments

```json
[
  {
    "name": "shardchannel",
    "type": "string",
    "optional": true,
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
