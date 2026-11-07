# XGROUP SETID

Sets the last-delivered ID of a consumer group.

## Syntax

```text
XGROUP SETID key group id|$ [ENTRIESREAD entries-read]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** -5

**ACL categories** STREAM

**Command flags** WRITE

## History

- 7.0.0 — Added the optional `entries_read` argument.

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
      "UPDATE"
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
