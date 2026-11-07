# OBJECT ENCODING

Returns the internal encoding of a Redis object.

## Syntax

```text
OBJECT ENCODING key
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
  "oneOf": [
    {
      "description": "key doesn't exist",
      "type": "null"
    },
    {
      "description": "encoding of the object",
      "type": "string"
    }
  ]
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
