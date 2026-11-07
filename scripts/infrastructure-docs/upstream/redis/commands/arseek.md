# ARSEEK

Sets the ARINSERT / ARRING cursor to a specific index.

## Syntax

```text
ARSEEK key index
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(1)

**Arity** 3

**ACL categories** ARRAY

**Command flags** WRITE, FAST

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "index",
    "type": "integer"
  }
]
```


## Reply schema

```json
{
  "description": "1 if the cursor was set, 0 if the key does not exist.",
  "type": "integer"
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
