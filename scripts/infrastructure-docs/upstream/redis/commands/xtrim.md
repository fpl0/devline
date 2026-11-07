# XTRIM

Deletes messages from the beginning of a stream.

## Syntax

```text
XTRIM key MAXLEN|MINID [=|~] threshold [LIMIT count] [KEEPREF|DELREF|ACKED]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(N), with N being the number of evicted entries. Constant times are very small however, since entries are organized in macro nodes containing multiple entries that can be released with a single deallocation.

**Arity** -4

**ACL categories** STREAM

**Command flags** WRITE

## History

- 6.2.0 — Added the `MINID` trimming strategy and the `LIMIT` option.
- 8.2.0 — Added the `KEEPREF`, `DELREF` and `ACKED` options.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "trim",
    "type": "block",
    "arguments": [
      {
        "name": "strategy",
        "type": "oneof",
        "arguments": [
          {
            "name": "maxlen",
            "type": "pure-token",
            "token": "MAXLEN"
          },
          {
            "name": "minid",
            "type": "pure-token",
            "token": "MINID",
            "since": "6.2.0"
          }
        ]
      },
      {
        "name": "operator",
        "type": "oneof",
        "optional": true,
        "arguments": [
          {
            "name": "equal",
            "type": "pure-token",
            "token": "="
          },
          {
            "name": "approximately",
            "type": "pure-token",
            "token": "~"
          }
        ]
      },
      {
        "name": "threshold",
        "type": "string"
      },
      {
        "token": "LIMIT",
        "name": "count",
        "type": "integer",
        "optional": true,
        "since": "6.2.0"
      },
      {
        "name": "condition",
        "type": "oneof",
        "optional": true,
        "arguments": [
          {
            "name": "keepref",
            "type": "pure-token",
            "token": "KEEPREF"
          },
          {
            "name": "delref",
            "type": "pure-token",
            "token": "DELREF"
          },
          {
            "name": "acked",
            "type": "pure-token",
            "token": "ACKED"
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
  "description": "The number of entries deleted from the stream.",
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
      "DELETE"
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
