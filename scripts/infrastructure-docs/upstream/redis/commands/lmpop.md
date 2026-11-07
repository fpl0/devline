# LMPOP

Returns multiple elements from a list after removing them. Deletes the list if the last element was popped.

## Syntax

```text
LMPOP numkeys key [key ...] LEFT|RIGHT [COUNT count]
```


**Available since** 7.0.0

**Group** list

**Time complexity** O(N+M) where N is the number of provided keys and M is the number of elements returned.

**Arity** -4

**ACL categories** LIST

**Command flags** WRITE

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
    "name": "where",
    "type": "oneof",
    "arguments": [
      {
        "name": "left",
        "type": "pure-token",
        "token": "LEFT"
      },
      {
        "name": "right",
        "type": "pure-token",
        "token": "RIGHT"
      }
    ]
  },
  {
    "token": "COUNT",
    "name": "count",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "anyOf": [
    {
      "description": "If no element could be popped.",
      "type": "null"
    },
    {
      "description": "List key from which elements were popped.",
      "type": "array",
      "minItems": 2,
      "maxItems": 2,
      "items": [
        {
          "description": "Name of the key from which elements were popped.",
          "type": "string"
        },
        {
          "description": "Array of popped elements.",
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "string"
          }
        }
      ]
    }
  ]
}
```


## Key specifications

```json
[
  {
    "flags": [
      "RW",
      "ACCESS",
      "DELETE"
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
