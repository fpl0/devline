# XGROUP CREATE

Creates a consumer group.

## Syntax

```text
XGROUP CREATE key group id|$ [MKSTREAM] [ENTRIESREAD entries-read]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** -5

**ACL categories** STREAM

**Command flags** WRITE, DENYOOM

## History

- 7.0.0 — Added the `entries_read` named argument.

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
    "name": "id-selector",
    "type": "oneof",
    "arguments": [
      {
        "name": "id",
        "type": "string"
      },
      {
        "name": "new-id",
        "type": "pure-token",
        "token": "$"
      }
    ]
  },
  {
    "token": "MKSTREAM",
    "name": "mkstream",
    "type": "pure-token",
    "optional": true
  },
  {
    "name": "entriesread",
    "display": "entries-read",
    "token": "ENTRIESREAD",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "const": "OK"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "INSERT"
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
