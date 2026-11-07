# GETSET

Returns the previous string value of a key after setting it to a new value.

## Syntax

```text
GETSET key value
```


**Available since** 1.0.0

**Group** string

**Time complexity** O(1)

**Arity** 3

**Deprecated since** 6.2.0

**Replacement** `SET` with the `!GET` argument

**ACL categories** STRING

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
    "name": "value",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "oneOf": [
    {
      "description": "The old value stored at the key.",
      "type": "string"
    },
    {
      "description": "The key does not exist.",
      "type": "null"
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
      "ACCESS",
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
