# XRANGE

Returns the messages from a stream within a range of IDs.

## Syntax

```text
XRANGE key start end [COUNT count]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(N) with N being the number of elements being returned. If N is constant (e.g. always asking for the first 10 elements with COUNT), you can consider it O(1).

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
    "name": "start",
    "type": "string"
  },
  {
    "name": "end",
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
  "description": "Stream entries with IDs matching the specified range.",
  "type": "array",
  "uniqueItems": true,
  "items": {
    "type": "array",
    "minItems": 2,
    "maxItems": 2,
    "items": [
      {
        "description": "Entry ID",
        "type": "string",
        "pattern": "[0-9]+-[0-9]+"
      },
      {
        "description": "Data",
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
