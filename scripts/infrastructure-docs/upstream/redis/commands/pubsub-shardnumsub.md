# PUBSUB SHARDNUMSUB

Returns the count of subscribers of shard channels.

## Syntax

```text
PUBSUB SHARDNUMSUB [shardchannel [shardchannel ...]]
```


**Available since** 7.0.0

**Group** pubsub

**Time complexity** O(N) for the SHARDNUMSUB subcommand, where N is the number of requested shard channels

**Arity** -2

**Command flags** PUBSUB, LOADING, STALE

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


## Reply schema

```json
{
  "description": "the number of subscribers per shard channel, each even element (including 0th) is channel name, each odd element is the number of subscribers",
  "type": "array"
}
```
