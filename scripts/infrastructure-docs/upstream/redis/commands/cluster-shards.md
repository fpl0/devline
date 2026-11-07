# CLUSTER SHARDS

Returns the mapping of cluster slots to shards.

## Syntax

```text
CLUSTER SHARDS (null)
```


**Available since** 7.0.0

**Group** cluster

**Time complexity** O(N) where N is the total number of cluster nodes

**Arity** 2

**Command flags** LOADING, STALE

## Reply schema

```json
{
  "description": "a nested list of a map of hash ranges and shard nodes describing individual shards",
  "type": "array",
  "items": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "slots": {
        "description": "an even number element array specifying the start and end slot numbers for slot ranges owned by this shard",
        "type": "array",
        "items": {
          "type": "integer"
        }
      },
      "nodes": {
        "description": "nodes that handle these slot ranges",
        "type": "array",
        "items": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "id": {
              "type": "string"
            },
            "port": {
              "type": "integer"
            },
            "tls-port": {
              "type": "integer"
            },
            "ip": {
              "type": "string"
            },
            "endpoint": {
              "type": "string"
            },
            "hostname": {
              "type": "string"
            },
            "role": {
              "oneOf": [
                {
                  "const": "master"
                },
                {
                  "const": "replica"
                }
              ]
            },
            "replication-offset": {
              "type": "integer"
            },
            "health": {
              "oneOf": [
                {
                  "const": "fail"
                },
                {
                  "const": "loading"
                },
                {
                  "const": "online"
                }
              ]
            }
          }
        }
      }
    }
  }
}
```
