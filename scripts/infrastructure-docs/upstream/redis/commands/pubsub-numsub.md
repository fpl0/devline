# PUBSUB NUMSUB

Returns a count of subscribers to channels.

## Syntax

```text
PUBSUB NUMSUB [channel [channel ...]]
```


**Available since** 2.8.0

**Group** pubsub

**Time complexity** O(N) for the NUMSUB subcommand, where N is the number of requested channels

**Arity** -2

**Command flags** PUBSUB, LOADING, STALE

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


## Reply schema

```json
{
  "description": "the number of subscribers per channel, each even element (including 0th) is channel name, each odd element is the number of subscribers",
  "type": "array"
}
```
