# XINFO CONSUMERS

Returns a list of the consumers in a consumer group.

## Syntax

```text
XINFO CONSUMERS key group
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** 4

**ACL categories** STREAM

**Command flags** READONLY

## History

- 7.2.0 — Added the `inactive` field, and changed the meaning of `idle`.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "group",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "Array list of consumers",
  "type": "array",
  "uniqueItems": true,
  "items": {
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "name": {
        "type": "string"
      },
      "pending": {
        "type": "integer"
      },
      "idle": {
        "type": "integer"
      },
      "inactive": {
        "type": "integer"
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
