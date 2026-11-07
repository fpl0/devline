# CLUSTER SLOT-STATS

Return an array of slot usage statistics for slots assigned to the current node.

## Syntax

```text
CLUSTER SLOT-STATS SLOTSRANGE start-slot end-slot|ORDERBY metric [LIMIT limit] [ASC|DESC]
```


**Available since** 8.2.0

**Group** cluster

**Time complexity** O(N) where N is the total number of slots based on arguments. O(N*log(N)) with ORDERBY subcommand.

**Arity** -4

**Command flags** STALE, LOADING

## Arguments

```json
[
  {
    "name": "filter",
    "type": "oneof",
    "arguments": [
      {
        "token": "SLOTSRANGE",
        "name": "slotsrange",
        "type": "block",
        "arguments": [
          {
            "name": "start-slot",
            "type": "integer"
          },
          {
            "name": "end-slot",
            "type": "integer"
          }
        ]
      },
      {
        "token": "ORDERBY",
        "name": "orderby",
        "type": "block",
        "arguments": [
          {
            "name": "metric",
            "type": "string"
          },
          {
            "token": "LIMIT",
            "name": "limit",
            "type": "integer",
            "optional": true
          },
          {
            "name": "order",
            "type": "oneof",
            "optional": true,
            "arguments": [
              {
                "name": "asc",
                "type": "pure-token",
                "token": "ASC"
              },
              {
                "name": "desc",
                "type": "pure-token",
                "token": "DESC"
              }
            ]
          }
        ]
      }
    ]
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "description": "Array of nested arrays, where the inner array element represents a slot and its respective usage statistics.",
  "items": {
    "type": "array",
    "description": "Array of size 2, where 0th index represents (int) slot and 1st index represents (map) usage statistics.",
    "minItems": 2,
    "maxItems": 2,
    "items": [
      {
        "description": "Slot Number.",
        "type": "integer"
      },
      {
        "type": "object",
        "description": "Map of slot usage statistics.",
        "additionalProperties": false,
        "properties": {
          "key-count": {
            "type": "integer"
          },
          "memory-bytes": {
            "type": "integer"
          },
          "cpu-usec": {
            "type": "integer"
          },
          "network-bytes-in": {
            "type": "integer"
          },
          "network-bytes-out": {
            "type": "integer"
          }
        }
      }
    ]
  }
}
```
