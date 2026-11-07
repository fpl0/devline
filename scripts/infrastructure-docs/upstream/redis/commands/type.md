# TYPE

Determines the type of value stored at a key.

## Syntax

```text
TYPE key
```


**Available since** 1.0.0

**Group** generic

**Time complexity** O(1)

**Arity** 2

**ACL categories** KEYSPACE

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
  "oneOf": [
    {
      "description": "Key doesn't exist",
      "type": "null"
    },
    {
      "description": "Type of the key",
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
