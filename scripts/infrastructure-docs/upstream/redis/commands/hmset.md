# HMSET

Sets the values of multiple fields.

## Syntax

```text
HMSET key field value [field value ...]
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(N) where N is the number of fields being set.

**Arity** -4

**Deprecated since** 4.0.0

**Replacement** `HSET` with multiple field-value pairs

**ACL categories** HASH

**Command flags** WRITE, DENYOOM, FAST

**Documentation flags** DEPRECATED

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "data",
    "type": "block",
    "multiple": true,
    "arguments": [
      {
        "name": "field",
        "type": "string"
      },
      {
        "name": "value",
        "type": "string"
      }
    ]
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
