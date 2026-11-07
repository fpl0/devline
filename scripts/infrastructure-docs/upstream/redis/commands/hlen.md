# HLEN

Returns the number of fields in a hash.

## Syntax

```text
HLEN key
```


**Available since** 2.0.0

**Group** hash

**Time complexity** O(1)

**Arity** 2

**ACL categories** HASH

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
  "type": "integer",
  "description": "Number of the fields in the hash, or 0 when the key does not exist."
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
