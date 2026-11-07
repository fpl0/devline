# HRANDFIELD

Returns one or more random fields from a hash.

## Syntax

```text
HRANDFIELD key [count [WITHVALUES]]
```


**Available since** 6.2.0

**Group** hash

**Time complexity** O(N) where N is the number of fields returned

**Arity** -2

**ACL categories** HASH

**Command flags** READONLY

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "options",
    "type": "block",
    "optional": true,
    "arguments": [
      {
        "name": "count",
        "type": "integer"
      },
      {
        "name": "withvalues",
        "token": "WITHVALUES",
        "type": "pure-token",
        "optional": true
      }
    ]
  }
]
```


## Reply schema

```json
{
  "anyOf": [
    {
      "description": "Key doesn't exist",
      "type": "null"
    },
    {
      "description": "A single random field. Returned in case `COUNT` was not used.",
      "type": "string"
    },
    {
      "description": "A list of fields. Returned in case `COUNT` was used.",
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    {
      "description": "Fields and their values. Returned in case `COUNT` and `WITHVALUES` were used. In RESP2 this is returned as a flat array.",
      "type": "array",
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "description": "Field",
            "type": "string"
          },
          {
            "description": "Value",
            "type": "string"
          }
        ]
      }
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
