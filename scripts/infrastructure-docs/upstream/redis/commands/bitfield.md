# BITFIELD

Performs arbitrary bitfield integer operations on strings.

## Syntax

```text
BITFIELD key [GET encoding offset|[OVERFLOW WRAP|SAT|FAIL] SET encoding offset value|INCRBY encoding offset increment [GET encoding offset|[OVERFLOW WRAP|SAT|FAIL] SET encoding offset value|INCRBY encoding offset increment ...]]
```


**Available since** 3.2.0

**Group** bitmap

**Time complexity** O(1) for each subcommand specified

**Arity** -2

**ACL categories** BITMAP

**Command flags** WRITE, DENYOOM

## Arguments

```json
[
  {
    "name": "key",
    "type": "key",
    "key_spec_index": 0
  },
  {
    "name": "operation",
    "type": "oneof",
    "multiple": true,
    "optional": true,
    "arguments": [
      {
        "token": "GET",
        "name": "get-block",
        "type": "block",
        "arguments": [
          {
            "name": "encoding",
            "type": "string"
          },
          {
            "name": "offset",
            "type": "integer"
          }
        ]
      },
      {
        "name": "write",
        "type": "block",
        "arguments": [
          {
            "token": "OVERFLOW",
            "name": "overflow-block",
            "type": "oneof",
            "optional": true,
            "arguments": [
              {
                "name": "wrap",
                "type": "pure-token",
                "token": "WRAP"
              },
              {
                "name": "sat",
                "type": "pure-token",
                "token": "SAT"
              },
              {
                "name": "fail",
                "type": "pure-token",
                "token": "FAIL"
              }
            ]
          },
          {
            "name": "write-operation",
            "type": "oneof",
            "arguments": [
              {
                "token": "SET",
                "name": "set-block",
                "type": "block",
                "arguments": [
                  {
                    "name": "encoding",
                    "type": "string"
                  },
                  {
                    "name": "offset",
                    "type": "integer"
                  },
                  {
                    "name": "value",
                    "type": "integer"
                  }
                ]
              },
              {
                "token": "INCRBY",
                "name": "incrby-block",
                "type": "block",
                "arguments": [
                  {
                    "name": "encoding",
                    "type": "string"
                  },
                  {
                    "name": "offset",
                    "type": "integer"
                  },
                  {
                    "name": "increment",
                    "type": "integer"
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
]
```


## Reply schema

```json
{
  "type": "array",
  "items": {
    "oneOf": [
      {
        "description": "The result of the subcommand at the same position",
        "type": "integer"
      },
      {
        "description": "In case OVERFLOW FAIL was given and overflows or underflows detected",
        "type": "null"
      }
    ]
  }
}
```


## Key specifications

```json
[
  {
    "notes": "This command allows both access and modification of the key",
    "flags": [
      "RW",
      "UPDATE",
      "ACCESS",
      "VARIABLE_FLAGS"
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
