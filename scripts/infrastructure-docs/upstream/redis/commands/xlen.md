# XLEN

Return the number of messages in a stream.

## Syntax

```text
XLEN key
```


**Available since** 5.0.0

**Group** stream

**Time complexity** O(1)

**Arity** 2

**ACL categories** STREAM

**Command flags** READONLY, FAST

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
  "description": "The number of entries of the stream at key",
  "type": "integer",
  "minimum": 0
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
