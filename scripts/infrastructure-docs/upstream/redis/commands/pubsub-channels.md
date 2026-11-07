# PUBSUB CHANNELS

Returns the active channels.

## Syntax

```text
PUBSUB CHANNELS [pattern]
```


**Available since** 2.8.0

**Group** pubsub

**Time complexity** O(N) where N is the number of active channels, and assuming constant time pattern matching (relatively short channels and patterns)

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
  "uniqueItems": true,
  "items": {
    "type": "string"
  }
}
```
