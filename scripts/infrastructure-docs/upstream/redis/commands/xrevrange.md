# XREVRANGE

Returns the messages from a stream within a range of IDs in reverse order.

## Syntax

```text
XREVRANGE key end start [COUNT count]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(N) with N being the number of elements returned. If N is constant (e.g. always asking for the first 10 elements with COUNT), you can consider it O(1).

**Arity** -4

**ACL categories** STREAM

**Command flags** READONLY

## History

- 6.2.0 — Added exclusive ranges.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "end",
    "type": "string"
  },
  {
    "name": "start",
    "type": "string"
  },
  {
    "token": "COUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "An array of the entries with IDs matching the specified range",
  "type": "array",
  "items": {
    "type": "array",
    "minItems": 2,
    "maxItems": 2,
    "items": [
      {
        "description": "Stream id",
        "type": "string",
        "pattern": "[0-9]+-[0-9]+"
      },
      {
        "description": "Array of field-value pairs",
        "type": "array",
        "items": {
          "type": "string"
        }
      }
    ]
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
        "pos": 1
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
