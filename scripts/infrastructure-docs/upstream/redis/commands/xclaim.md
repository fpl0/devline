# XCLAIM

Changes, or acquires, ownership of a message in a consumer group, as if the message was delivered a consumer group member.

## Syntax

```text
XCLAIM key group consumer min-idle-time id [id ...] [IDLE ms] [TIME unix-time-milliseconds] [RETRYCOUNT count] [FORCE] [JUSTID] [LASTID lastid]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(log N) with N being the number of messages in the PEL of the consumer group.

**Arity** -6

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
    "name": "consumer",
    "type": "string"
  },
  {
    "name": "min-idle-time",
    "type": "string"
  },
  {
    "name": "ID",
    "type": "string",
    "multiple": true
  },
  {
    "token": "IDLE",
    "name": "ms",
    "type": "integer",
    "optional": true
  },
  {
    "token": "TIME",
    "name": "unix-time-milliseconds",
    "type": "unix-time",
    "optional": true
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
  },
  {
    "name": "justid",
    "token": "JUSTID",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "lastid",
    "token": "LASTID",
    "type": "string",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "Stream entries with IDs matching the specified range.",
  "anyOf": [
    {
      "description": "If JUSTID option is specified, return just an array of IDs of messages successfully claimed",
      "type": "array",
      "items": {
        "description": "Entry ID",
        "type": "string",
        "pattern": "[0-9]+-[0-9]+"
      }
    },
    {
      "description": "array of stream entries that contains each entry as an array of 2 elements, the Entry ID and the entry data itself",
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
  ]
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
