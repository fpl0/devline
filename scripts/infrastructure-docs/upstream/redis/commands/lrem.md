# LREM

Removes elements from a list. Deletes the list if the last element was removed.

## Syntax

```text
LREM key count element
```


**Available since** 1.0.0

**Group** list

**Time complexity** O(N+M) where N is the length of the list and M is the number of elements removed.

**Arity** 4

**ACL categories** LIST

**Command flags** WRITE

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "count",
    "type": "integer"
  },
  {
    "name": "element",
    "type": "string"
  }
]
```


## Reply schema

```json
{
  "description": "The number of removed elements.",
  "type": "integer",
  "minimum": 0
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "DELETE"
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
