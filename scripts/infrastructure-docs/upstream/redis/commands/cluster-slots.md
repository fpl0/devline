# CLUSTER SLOTS

Returns the mapping of cluster slots to nodes.

## Syntax

```text
CLUSTER SLOTS (null)
```


**Available since** 3.0.0

**Group** cluster

**Time complexity** O(N) where N is the total number of Cluster nodes

**Arity** 2

**Deprecated since** 7.0.0

**Replacement** `CLUSTER SHARDS`

**Command flags** LOADING, STALE

**Documentation flags** DEPRECATED

## History

- 4.0.0 — Added node IDs.
- 7.0.0 — Added additional networking metadata field.

## Reply schema

```json
{
  "description": "nested list of slot ranges with networking information",
  "type": "array",
  "items": {
    "type": "array",
    "minItems": 3,
    "maxItems": 4294967295,
    "items": [
      {
        "description": "start slot number",
        "type": "integer"
      },
      {
        "description": "end slot number",
        "type": "integer"
      },
      {
        "type": "array",
        "description": "Master node for the slot range",
        "minItems": 4,
        "maxItems": 4,
        "items": [
          {
            "description": "endpoint description",
            "oneOf": [
              {
                "description": "hostname or ip",
                "type": "string"
              },
              {
                "description": "unknown type",
                "type": "null"
              }
            ]
          },
          {
            "description": "port",
            "type": "integer"
          },
          {
            "description": "node name",
            "type": "string"
          },
          {
            "description": "array of node descriptions",
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "hostname": {
                "type": "string"
              },
              "ip": {
                "type": "string"
              }
            }
          }
        ]
      }
    ],
    "additionalItems": {
      "type": "array",
      "description": "Replica node for the slot range",
      "minItems": 4,
      "maxItems": 4,
      "items": [
        {
          "description": "endpoint description",
          "oneOf": [
            {
              "description": "hostname or ip",
              "type": "string"
            },
            {
              "description": "unknown type",
              "type": "null"
            }
          ]
        },
        {
          "description": "port",
          "type": "integer"
        },
        {
          "description": "node name",
          "type": "string"
        },
        {
          "description": "array of node descriptions",
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "hostname": {
              "type": "string"
            },
            "ip": {
              "type": "string"
            }
          }
        }
      ]
    }
  }
}
```
