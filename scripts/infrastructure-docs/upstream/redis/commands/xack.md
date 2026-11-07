# XACK

Returns the number of messages that were successfully acknowledged by the consumer group member of a stream.

## Syntax

```text
XACK key group id [id ...]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1) for each message ID processed.

**Arity** -4

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
    "name": "ID",
    "type": "string",
    "multiple": true
  }
]
```


## Reply schema

```json
{
  "description": "The command returns the number of messages successfully acknowledged. Certain message IDs may no longer be part of the PEL (for example because they have already been acknowledged), and XACK will not count them as successfully acknowledged.",
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
