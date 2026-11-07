# XNACK

Releases claimed messages back to the group's PEL without acknowledging them, making them available for re-delivery.

## Syntax

```text
XNACK key group SILENT|FAIL|FATAL IDS numids id [id ...] [RETRYCOUNT count] [FORCE]
```


**Available since** 8.8.0

**Group** stream

**Time complexity** O(1) for each message ID processed.

**Arity** -7

**ACL categories** STREAM

**Command flags** WRITE, FAST

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
  },
  {
    "name": "mode",
    "type": "oneof",
    "arguments": [
      {
        "name": "silent",
        "type": "pure-token",
        "token": "SILENT"
      },
      {
        "name": "fail",
        "type": "pure-token",
        "token": "FAIL"
      },
      {
        "name": "fatal",
        "type": "pure-token",
        "token": "FATAL"
      }
    ]
  },
  {
    "name": "ids",
    "token": "IDS",
    "type": "block",
    "arguments": [
      {
        "name": "numids",
        "type": "integer"
      },
      {
        "name": "id",
        "type": "string",
        "multiple": true
      }
    ]
  },
  {
    "token": "RETRYCOUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  },
  {
    "name": "force",
    "token": "FORCE",
    "type": "pure-token",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "The number of messages successfully NACKed (released back to the group PEL).",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "UPDATE"
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
