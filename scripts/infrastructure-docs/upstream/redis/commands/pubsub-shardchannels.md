# PUBSUB SHARDCHANNELS

Returns the active shard channels.

## Syntax

```text
PUBSUB SHARDCHANNELS [pattern]
```


**Available since** 7.0.0

**Group** pubsub

**Time complexity** O(N) where N is the number of active shard channels, and assuming constant time pattern matching (relatively short shard channels).

**Arity** -2

**Command flags** PUBSUB, LOADING, STALE

## Arguments

```json
[
  {
    "name": "pattern",
    "type": "pattern",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "a list of active channels, optionally matching the specified pattern",
  "type": "array",
  "items": {
    "type": "string"
  },
  "uniqueItems": true
}
```
