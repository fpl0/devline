# ARGREP

Searches array elements in a range using textual predicates.

## Syntax

```text
ARGREP key start end EXACT string|MATCH string|GLOB pattern|RE pattern [EXACT string|MATCH string|GLOB pattern|RE pattern ...] [AND|OR|LIMIT limit|WITHVALUES|NOCASE [AND|OR|LIMIT limit|WITHVALUES|NOCASE ...]]
```


**Available since** 8.8.0

**Group** array

**Time complexity** O(P * C) where P is the number of visited positions in touched slices and C is the cost of evaluating the predicates on one existing element.

**Arity** -6

**ACL categories** ARRAY

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
    "name": "start",
    "type": "string"
  },
  {
    "name": "end",
    "type": "string"
  },
  {
    "name": "predicate",
    "type": "oneof",
    "multiple": true,
    "arguments": [
      {
        "name": "exact",
        "type": "block",
        "arguments": [
          {
            "name": "exact",
            "type": "pure-token",
            "token": "EXACT"
          },
          {
            "name": "string",
            "type": "string"
          }
        ]
      },
      {
        "name": "match",
        "type": "block",
        "arguments": [
          {
            "name": "match",
            "type": "pure-token",
            "token": "MATCH"
          },
          {
            "name": "string",
            "type": "string"
          }
        ]
      },
      {
        "name": "glob",
        "type": "block",
        "arguments": [
          {
            "name": "glob",
            "type": "pure-token",
            "token": "GLOB"
          },
          {
            "name": "pattern",
            "type": "string"
          }
        ]
      },
      {
        "name": "re",
        "type": "block",
        "arguments": [
          {
            "name": "re",
            "type": "pure-token",
            "token": "RE"
          },
          {
            "name": "pattern",
            "type": "string"
          }
        ]
      }
    ]
  },
  {
    "name": "options",
    "type": "oneof",
    "optional": true,
    "multiple": true,
    "arguments": [
      {
        "name": "and",
        "type": "pure-token",
        "token": "AND"
      },
      {
        "name": "or",
        "type": "pure-token",
        "token": "OR"
      },
      {
        "name": "limit",
        "type": "integer",
        "token": "LIMIT"
      },
      {
        "name": "withvalues",
        "type": "pure-token",
        "token": "WITHVALUES"
      },
      {
        "name": "nocase",
        "type": "pure-token",
        "token": "NOCASE"
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
      "description": "Array of matching indexes.",
      "type": "array",
      "items": {
        "type": "integer",
        "description": "Index of a matching element"
      }
    },
    {
      "description": "Array of [index, value] pairs. Returned in case `WITHVALUES` was used.",
      "type": "array",
      "items": {
        "type": "array",
        "minItems": 2,
        "maxItems": 2,
        "items": [
          {
            "type": "integer",
            "description": "Index of a matching element"
          },
          {
            "type": "string",
            "description": "Value at that index"
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
