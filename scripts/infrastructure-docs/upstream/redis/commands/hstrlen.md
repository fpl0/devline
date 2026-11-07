# HSTRLEN

Returns the length of the value of a field.

## Syntax

```text
HSTRLEN key field
```


**Available since** 3.2.0

**Group** hash

**Time complexity** O(1)

**Arity** 3

**ACL categories** HASH

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "field",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "type": "integer",
  "description": "String length of the value associated with the field, or zero when the field is not present in the hash or key does not exist at all.",
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
