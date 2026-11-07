# ZPOPMIN

Returns the lowest-scoring members from a sorted set after removing them. Deletes the sorted set if the last member was popped.

## Syntax

```text
ZPOPMIN key [count]
```


**Available since** 5.0.0

**Group** sorted_set

**Time complexity** O(log(N)*M) with N being the number of elements in the sorted set, and M being the number of elements popped.

**Arity** -2

**ACL categories** SORTEDSET

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
      "type": "array",
      "description": "List of popped elements and scores when 'COUNT' isn't specified.",
      "minItems": 2,
      "maxItems": 2,
      "items": [
        {
          "type": "string",
          "description": "Popped element."
        },
        {
          "type": "number",
          "description": "Score."
        }
      ]
    },
    {
      "type": "array",
      "description": "List of popped elements and scores when 'COUNT' is specified.",
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "type": "string",
            "description": "Popped element."
          },
          {
            "type": "number",
            "description": "Score."
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
      "range": {
        "lastkey": 0,
        "step": 1,
        "limit": 0
      }
    }
  }
]
```
