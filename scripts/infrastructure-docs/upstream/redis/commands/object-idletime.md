# OBJECT IDLETIME

Returns the time since the last access to a Redis object.

## Syntax

```text
OBJECT IDLETIME key
```


**Available since** 2.2.3

**Group** generic

**Time complexity** O(1)

**Arity** 3

**ACL categories** KEYSPACE

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  }
]
```


## Reply schema

```json
{
  "description": "the idle time in seconds",
  "type": "integer"
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RO"
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
