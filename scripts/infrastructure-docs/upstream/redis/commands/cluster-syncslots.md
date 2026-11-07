# CLUSTER SYNCSLOTS

Internal command for atomic slot migration protocol between cluster nodes.

## Syntax

```text
CLUSTER SYNCSLOTS SYNC task-id start-slot end-slot [start-slot end-slot ...]|RDBCHANNEL task-id|SNAPSHOT-EOF|STREAM-EOF|ACK state offset|FAIL error|CONF option [option ...] value [value ...]
```


**Available since** 8.4.0

**Group** cluster

**Time complexity** O(1)

**Arity** -3

**Command flags** NO_ASYNC_LOADING, ADMIN, STALE

## Arguments

```json
[
  {
    "name": "subcommand",
    "type": "oneof",
    "arguments": [
      {
        "name": "sync",
        "token": "SYNC",
        "type": "block",
        "arguments": [
          {
            "name": "task-id",
            "type": "string"
          },
          {
            "name": "slot-range",
            "type": "block",
            "multiple": true,
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
          }
        ]
      },
      {
        "token": "RDBCHANNEL",
        "name": "task-id",
        "type": "string"
      },
      {
        "name": "snapshot-eof",
        "token": "SNAPSHOT-EOF",
        "type": "pure-token"
      },
      {
        "name": "stream-eof",
        "token": "STREAM-EOF",
        "type": "pure-token"
      },
      {
        "name": "ack",
        "token": "ACK",
        "type": "block",
        "arguments": [
          {
            "name": "state",
            "type": "string"
          },
          {
            "name": "offset",
            "type": "integer"
          }
        ]
      },
      {
        "token": "FAIL",
        "name": "error",
        "type": "string"
      },
      {
        "name": "conf",
        "token": "CONF",
        "type": "block",
        "arguments": [
          {
            "name": "option",
            "type": "string",
            "multiple": true
          },
          {
            "name": "value",
            "type": "string",
            "multiple": true
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
  "oneOf": [
    {
      "description": "Reply to CLUSTER SYNCSLOTS SYNC, returns special RDB channel sync response.",
      "const": "RDBCHANNELSYNCSLOTS"
    },
    {
      "description": "Reply to CLUSTER SYNCSLOTS CONF and other subcommands.",
      "const": "OK"
    }
  ]
}
```
