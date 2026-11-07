# PUBSUB NUMPAT

Returns a count of unique pattern subscriptions.

## Syntax

```text
PUBSUB NUMPAT (null)
```


**Available since** 2.8.0

**Group** pubsub

**Time complexity** O(1)

**Arity** 2

**Command flags** PUBSUB, LOADING, STALE

## Reply schema

```json
{
  "description": "the number of patterns all the clients are subscribed to",
  "type": "integer",
  "minimum": 0
}
```
