# DIGEST

Returns the XXH3 hash of a string value.

## Syntax

```text
DIGEST key
```


**Available since** 8.4.0

**Group** string

**Time complexity** O(N) where N is the length of the string value.

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
      "description": "The XXH3 64-bit hash of the string value as a signed integer.",
      "type": "string"
    },
    {
      "description": "Key does not exist",
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
