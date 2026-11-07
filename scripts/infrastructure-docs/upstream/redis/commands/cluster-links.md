# CLUSTER LINKS

Returns a list of all TCP links to and from peer nodes.

## Syntax

```text
CLUSTER LINKS (null)
```


**Available since** 7.0.0

**Group** cluster

**Time complexity** O(N) where N is the total number of Cluster nodes

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "description": "an array of cluster links and their attributes",
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "direction": {
        "description": "This link is established by the local node _to_ the peer, or accepted by the local node _from_ the peer.",
        "oneOf": [
          {
            "description": "connection initiated from peer",
            "const": "from"
          },
          {
            "description": "connection initiated to peer",
            "const": "to"
          }
        ]
      },
      "node": {
        "description": "the node id of the peer",
        "type": "string"
      },
      "create-time": {
        "description": "unix time creation time of the link. (In the case of a _to_ link, this is the time when the TCP link is created by the local node, not the time when it is actually established.)",
        "type": "integer"
      },
      "events": {
        "description": "events currently registered for the link. r means readable event, w means writable event",
        "type": "string"
      },
      "send-buffer-allocated": {
        "description": "allocated size of the link's send buffer, which is used to buffer outgoing messages toward the peer",
        "type": "integer"
      },
      "send-buffer-used": {
        "description": "size of the portion of the link's send buffer that is currently holding data(messages)",
        "type": "integer"
      }
    },
    "additionalProperties": false
  }
}
```
