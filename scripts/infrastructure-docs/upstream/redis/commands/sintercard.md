# SINTERCARD

Returns the number of members of the intersect of multiple sets.

## Syntax

```text
SINTERCARD numkeys key [key ...] [LIMIT limit]
```


**Available since** 7.0.0

**Group** set

**Time complexity** O(N*M) worst case where N is the cardinality of the smallest set and M is the number of sets.

**Arity** -3

**ACL categories** SET

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "numkeys",
    "type": "integer"
  },
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0,
    "multiple": true
  },
  {
    "token": "LIMIT",
    "name": "limit",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "Number of the elements in the resulting intersection.",
  "type": "integer",
  "minimum": 0
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
      "keynum": {
        "keynumidx": 0,
        "firstkey": 1,
        "step": 1
      }
    }
  }
]
```
