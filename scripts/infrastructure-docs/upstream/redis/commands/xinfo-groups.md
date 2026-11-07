# XINFO GROUPS

Returns a list of the consumer groups of a stream.

## Syntax

```text
XINFO GROUPS key
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** 3

**ACL categories** STREAM

**Command flags** READONLY

## History

- 7.0.0 — Added the `entries-read` and `lag` fields

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "items": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "name": {
        "type": "string"
      },
      "consumers": {
        "type": "integer"
      },
      "pending": {
        "type": "integer"
      },
      "last-delivered-id": {
        "type": "string",
        "pattern": "[0-9]+-[0-9]+"
      },
      "entries-read": {
        "oneOf": [
          {
            "type": "null"
          },
          {
            "type": "integer"
          }
        ]
      },
      "lag": {
        "oneOf": [
          {
            "type": "null"
          },
          {
            "type": "integer"
          }
        ]
      }
    }
  }
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO",
      "ACCESS"
    ],
    "begin_search": {
      "index": {
        "pos": 2
      }
    },
    "find_keys": {
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
