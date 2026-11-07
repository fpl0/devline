# XSETID

An internal command for replicating stream values.

## Syntax

```text
XSETID key last-id [ENTRIESADDED entries-added] [MAXDELETEDID max-deleted-id]
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** -3

**ACL categories** STREAM

**Command flags** WRITE, DENYOOM, FAST

## History

- 7.0.0 — Added the `entries_added` and `max_deleted_entry_id` arguments.

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "last-id",
    "type": "string"
  },
  {
    "name": "entries-added",
    "token": "ENTRIESADDED",
    "type": "integer",
    "optional": true,
    "since": "7.0.0"
  },
  {
    "name": "max-deleted-id",
    "token": "MAXDELETEDID",
    "type": "string",
    "optional": true,
    "since": "7.0.0"
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
