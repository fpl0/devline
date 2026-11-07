# GET

Returns the string value of a key.

## Syntax

```text
GET key
```


**Available since** 1.0.0

**Group** string

**Time complexity** O(1)

**Arity** 2

**ACL categories** STRING

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
      "description": "The value of the key.",
      "type": "string"
    },
    {
      "description": "Key does not exist.",
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
      "RO",
      "ACCESS"
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
