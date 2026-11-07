# ARLEN

Returns the length of an array (max index + 1).

## Syntax

```text
ARLEN key
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(1)

**Arity** 2

**ACL categories** ARRAY

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
  "description": "The length of the array (max index + 1), or 0 if key does not exist.",
  "type": "integer"
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
