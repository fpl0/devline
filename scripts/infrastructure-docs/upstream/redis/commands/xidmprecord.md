# XIDMPRECORD

An internal command for setting IDMP metadata on an existing stream message.

## Syntax

```text
XIDMPRECORD key pid iid stream-id
```


**Available since** 8.6.2

**Group** stream

**Time complexity** O(1)

**Arity** 5

**ACL categories** STREAM

**Command flags** WRITE, DENYOOM, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "pid",
    "type": "string"
  },
  {
    "name": "iid",
    "type": "string"
  },
  {
    "name": "stream-id",
    "type": "string"
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
